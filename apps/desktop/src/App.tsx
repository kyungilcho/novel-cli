import { useEffect, useMemo, useState } from "react";
import { defaultWorkspaceRoot, openProject } from "./lib/api/projectApi";
import { createFile, listFiles, readFile, writeFile } from "./lib/api/fileApi";
import {
    checkoutSnapshot,
    commitSnapshot,
    fetchLog,
    fetchRepoState,
    initRepo,
} from "./lib/api/vcsApi";
import type { ProjectInfo } from "./lib/api/projectApi";
import type { FileEntry } from "./lib/api/fileApi";
import type { RepoState, VersionNode } from "./lib/api/vcsApi";

type ActivityView = "explorer" | "search" | "history" | "lab";

type CursorPosition = {
    line: number;
    col: number;
};

type ActivityItem = {
    id: ActivityView;
    title: string;
    icon: string;
    hasDot?: boolean;
};

const ACTIVITY_ITEMS: ActivityItem[] = [
    { id: "explorer", title: "Explorer", icon: "folder_copy" },
    { id: "search", title: "Search", icon: "search" },
    { id: "history", title: "History", icon: "history", hasDot: true },
    { id: "lab", title: "Lab", icon: "science" },
];

function cursorFromText(text: string, offset: number): CursorPosition {
    const safeOffset = Math.max(0, Math.min(offset, text.length));

    let line = 1;
    let col = 1;

    for (let i = 0; i < safeOffset; i += 1) {
        if (text.charCodeAt(i) === 10) {
            line += 1;
            col = 1;
        } else {
            col += 1;
        }
    }

    return { line, col };
}

function inferLanguage(path: string): string {
    if (path.endsWith(".story")) return "Story Markdown";
    if (path.endsWith(".md")) return "Markdown";
    if (path.endsWith(".json")) return "JSON";
    if (path.endsWith(".txt")) return "Plain Text";
    return "Text";
}

function basename(path: string): string {
    const parts = path.split(/[\\/]/).filter(Boolean);
    return parts.length > 0 ? parts[parts.length - 1] : path;
}

function iconNameForPath(path: string, isDir: boolean): string {
    if (isDir) return "folder";
    if (path.endsWith(".json")) return "data_object";
    if (path.endsWith(".md")) return "description";
    return "article";
}

function iconToneClass(path: string, isDir: boolean): string {
    if (isDir) return "tone-folder";
    if (path.endsWith(".json")) return "tone-json";
    if (path.endsWith(".md")) return "tone-md";
    return "tone-story";
}

function formatNodeTime(unixMs: number): string {
    const d = new Date(unixMs);
    return d.toLocaleString();
}

export default function App() {
    const [inputRoot, setInputRoot] = useState("");
    const [project, setProject] = useState<ProjectInfo | null>(null);
    const [files, setFiles] = useState<FileEntry[]>([]);
    const [selectedPath, setSelectedPath] = useState("");
    const [content, setContent] = useState("");
    const [savedContent, setSavedContent] = useState("");
    const [error, setError] = useState("");
    const [newFileName, setNewFileName] = useState("");
    const [openTabs, setOpenTabs] = useState<string[]>([]);
    const [activeView, setActiveView] = useState<ActivityView>("explorer");
    const [sidebarOpen, setSidebarOpen] = useState(true);
    const [cursor, setCursor] = useState<CursorPosition>({ line: 1, col: 1 });
    const [vcsState, setVcsState] = useState<RepoState | null>(null);
    const [logNodes, setLogNodes] = useState<VersionNode[]>([]);
    const [commitMessage, setCommitMessage] = useState("");
    const [vcsBusy, setVcsBusy] = useState(false);

    const resetEditor = () => {
        setSelectedPath("");
        setContent("");
        setSavedContent("");
        setOpenTabs([]);
        setCursor({ line: 1, col: 1 });
    };

    const refreshWorkspaceFiles = async (rootPath: string) => {
        const list = await listFiles(rootPath, ".");
        setFiles(list);

        const fileSet = new Set(list.filter((f) => !f.is_dir).map((f) => f.path));
        setOpenTabs((prev) => prev.filter((p) => fileSet.has(p)));

        if (selectedPath && !fileSet.has(selectedPath)) {
            setSelectedPath("");
            setContent("");
            setSavedContent("");
            setCursor({ line: 1, col: 1 });
        }

        return list;
    };

    const refreshVcs = async (rootPath: string) => {
        await initRepo(rootPath);
        const [state, nodes] = await Promise.all([fetchRepoState(rootPath), fetchLog(rootPath)]);
        setVcsState(state);
        setLogNodes(nodes);
    };

    const loadProject = async (rootPath: string) => {
        const p = await openProject(rootPath.trim());
        setProject(p);
        resetEditor();
        await refreshWorkspaceFiles(p.root);
        await refreshVcs(p.root);
    };

    useEffect(() => {
        let cancelled = false;

        const initDefaultRoot = async () => {
            try {
                const root = await defaultWorkspaceRoot();
                if (!cancelled) {
                    setInputRoot(root);
                    await loadProject(root);
                }
            } catch (e) {
                if (!cancelled) {
                    setError(String(e));
                }
            }
        };

        void initDefaultRoot();

        return () => {
            cancelled = true;
        };
    }, []);

    const onUseIsolatedRoot = async () => {
        setError("");
        try {
            const root = await defaultWorkspaceRoot();
            setInputRoot(root);
            await loadProject(root);
        } catch (e) {
            setError(String(e));
        }
    };

    const onOpenProject = async () => {
        setError("");
        try {
            await loadProject(inputRoot);
        } catch (e) {
            setError(String(e));
        }
    };

    const onRefreshFiles = async () => {
        if (!project) return;
        setError("");
        try {
            await refreshWorkspaceFiles(project.root);
        } catch (e) {
            setError(String(e));
        }
    };

    const onRefreshVcs = async () => {
        if (!project) return;
        setError("");
        setVcsBusy(true);
        try {
            await refreshVcs(project.root);
        } catch (e) {
            setError(String(e));
        } finally {
            setVcsBusy(false);
        }
    };

    const onRead = async (path: string) => {
        if (!project) return;

        setError("");
        try {
            const text = await readFile(project.root, path);
            setSelectedPath(path);
            setContent(text);
            setSavedContent(text);
            setCursor({ line: 1, col: 1 });
            setOpenTabs((prev) => (prev.includes(path) ? prev : [...prev, path]));
            setSidebarOpen(false);
        } catch (e) {
            setError(String(e));
        }
    };

    const onSave = async () => {
        if (!project || !selectedPath) return;
        setError("");
        try {
            await writeFile(project.root, selectedPath, content);
            // Re-read after write so UI state is always aligned with disk content.
            const persisted = await readFile(project.root, selectedPath);
            setContent(persisted);
            setSavedContent(persisted);
        } catch (e) {
            setError(String(e));
        }
    };

    useEffect(() => {
        const onKeyDown = (e: KeyboardEvent) => {
            const isSaveKey = (e.metaKey || e.ctrlKey) && e.key.toLowerCase() === "s";
            if (!isSaveKey) return;

            e.preventDefault();
            void onSave();
        };

        window.addEventListener("keydown", onKeyDown);
        return () => window.removeEventListener("keydown", onKeyDown);
    }, [onSave]);

    const onCreateFile = async () => {
        if (!project || !newFileName.trim()) return;
        setError("");
        try {
            const createdPath = await createFile(project.root, ".", newFileName.trim());
            setNewFileName("");
            await refreshWorkspaceFiles(project.root);
            // Avoid async read race: new file content is known(empty) at creation time.
            setSelectedPath(createdPath);
            setContent("");
            setSavedContent("");
            setCursor({ line: 1, col: 1 });
            setOpenTabs((prev) => (prev.includes(createdPath) ? prev : [...prev, createdPath]));
            setSidebarOpen(false);
        } catch (e) {
            setError(String(e));
        }
    };

    const onCommitSnapshot = async () => {
        if (!project) return;
        const message = commitMessage.trim();
        if (!message) {
            setError("Commit message is required.");
            return;
        }

        setError("");
        setVcsBusy(true);
        try {
            if (isDirty && selectedPath) {
                await onSave();
            }

            await commitSnapshot(project.root, message);
            setCommitMessage("");
            await refreshVcs(project.root);
            await refreshWorkspaceFiles(project.root);
        } catch (e) {
            setError(String(e));
        } finally {
            setVcsBusy(false);
        }
    };

    const onCheckoutSnapshot = async (nodeId: string) => {
        if (!project) return;

        if (isDirty) {
            const ok = window.confirm(
                "Unsaved edits will be overwritten by checkout. Continue?",
            );
            if (!ok) return;
        }

        setError("");
        setVcsBusy(true);
        try {
            await checkoutSnapshot(project.root, nodeId);
            const list = await refreshWorkspaceFiles(project.root);

            const fileSet = new Set(list.filter((f) => !f.is_dir).map((f) => f.path));
            if (selectedPath && fileSet.has(selectedPath)) {
                const nextText = await readFile(project.root, selectedPath);
                setContent(nextText);
                setSavedContent(nextText);
                setCursor({ line: 1, col: 1 });
            } else {
                resetEditor();
            }

            await refreshVcs(project.root);
        } catch (e) {
            setError(String(e));
        } finally {
            setVcsBusy(false);
        }
    };

    const onCloseTab = (path: string) => {
        const idx = openTabs.indexOf(path);
        if (idx < 0) return;

        const nextTabs = openTabs.filter((p) => p !== path);
        setOpenTabs(nextTabs);

        if (selectedPath === path) {
            const fallback = nextTabs[idx] ?? nextTabs[idx - 1] ?? "";
            if (fallback) {
                void onRead(fallback);
            } else {
                setSelectedPath("");
                setContent("");
                setSavedContent("");
                setCursor({ line: 1, col: 1 });
            }
        }
    };

    const onEditorChange = (nextText: string, selectionStart: number) => {
        setContent(nextText);
        setCursor(cursorFromText(nextText, selectionStart));
    };

    const onEditorSelect = (text: string, selectionStart: number) => {
        setCursor(cursorFromText(text, selectionStart));
    };

    const onSelectActivity = (view: ActivityView) => {
        const isSame = view === activeView;
        setActiveView(view);

        if (view === "explorer") {
            setSidebarOpen((prev) => (isSame ? !prev : true));
        } else {
            setSidebarOpen(true);
        }
    };

    const sortedFiles = useMemo(() => {
        return [...files].sort((a, b) => {
            if (a.is_dir !== b.is_dir) return a.is_dir ? -1 : 1;
            return a.path.localeCompare(b.path);
        });
    }, [files]);

    const lineCount = useMemo(() => Math.max(content.split("\n").length, 1), [content]);
    const lineNumbers = useMemo(
        () => Array.from({ length: lineCount }, (_, idx) => idx + 1),
        [lineCount],
    );

    const minimapRows = useMemo(() => {
        const lines = content.length > 0 ? content.split("\n") : ["open a file to start writing"];
        return lines.slice(0, 120).map((line, idx) => {
            const base = Math.max(8, Math.min(92, line.trim().length * 2));
            return Math.max(10, Math.min(96, base + (idx % 5)));
        });
    }, [content]);

    const breadcrumbs = selectedPath.split(/[\\/]/).filter(Boolean);
    const isDirty = selectedPath !== "" && content !== savedContent;
    const language = selectedPath ? inferLanguage(selectedPath) : "Plain Text";
    const minimapViewportTop = lineCount > 1 ? Math.min(92, ((cursor.line - 1) / (lineCount - 1)) * 92) : 0;
    const headShort = vcsState?.head ? vcsState.head.slice(0, 8) : "-";

    return (
        <main className="app-shell" data-sidebar={sidebarOpen ? "open" : "closed"}>
            <aside className="activity-bar">
                <div className="activity-list">
                    {ACTIVITY_ITEMS.map((item) => (
                        <button
                            key={item.id}
                            type="button"
                            className={`activity-btn ${activeView === item.id ? "is-active" : ""}`}
                            onClick={() => onSelectActivity(item.id)}
                            title={item.title}
                        >
                            <span className="material-symbols-outlined activity-icon">{item.icon}</span>
                            {item.hasDot && <span className="activity-dot" />}
                        </button>
                    ))}

                    <div className="activity-spacer" />

                    <button type="button" className="activity-btn" title="Account">
                        <span className="material-symbols-outlined activity-icon">account_circle</span>
                    </button>
                    <button type="button" className="activity-btn" title="Settings">
                        <span className="material-symbols-outlined activity-icon">settings</span>
                    </button>
                </div>
            </aside>

            <aside className="explorer-panel">
                <div className="explorer-header">
                    {activeView === "history" ? "Source Control" : "Explorer"}
                </div>

                {activeView === "history" ? (
                    <>
                        <div className="explorer-section">
                            <span className="material-symbols-outlined section-chevron">chevron_right</span>
                            <span className="section-title">SNAPSHOTS</span>
                        </div>
                        <div className="history-controls">
                            <div className="history-stats mono">
                                <span>{`HEAD: ${vcsState?.head ? vcsState.head.slice(0, 8) : "-"}`}</span>
                                <span>{`COMMITS: ${vcsState?.node_count ?? 0}`}</span>
                            </div>
                            <textarea
                                className="ui-textarea history-message"
                                placeholder="snapshot message..."
                                value={commitMessage}
                                onChange={(e) => setCommitMessage(e.currentTarget.value)}
                                disabled={!project || vcsBusy}
                                rows={3}
                            />
                            <div className="explorer-controls-row">
                                <button
                                    type="button"
                                    className="ui-button is-primary"
                                    onClick={() => void onCommitSnapshot()}
                                    disabled={!project || vcsBusy || !commitMessage.trim()}
                                >
                                    Commit
                                </button>
                                <button
                                    type="button"
                                    className="ui-button"
                                    onClick={() => void onRefreshVcs()}
                                    disabled={!project || vcsBusy}
                                >
                                    Refresh Log
                                </button>
                            </div>
                        </div>
                        <div className="explorer-scroll">
                            {logNodes.length === 0 && (
                                <div className="tree-row is-disabled">No snapshots yet</div>
                            )}
                            {logNodes.map((node) => {
                                const isHead = vcsState?.head === node.id;
                                return (
                                    <div key={node.id} className={`history-item${isHead ? " is-head" : ""}`}>
                                        <div className="history-item-top">
                                            <span className="history-item-id mono">{node.id.slice(0, 8)}</span>
                                            <button
                                                type="button"
                                                className="ui-button history-item-btn"
                                                onClick={() => void onCheckoutSnapshot(node.id)}
                                                disabled={!project || vcsBusy || isHead}
                                            >
                                                {isHead ? "Current" : "Checkout"}
                                            </button>
                                        </div>
                                        <div className="history-item-message">{node.message}</div>
                                        <div className="history-item-meta mono">
                                            {formatNodeTime(node.created_at_unix_ms)}
                                        </div>
                                    </div>
                                );
                            })}
                        </div>
                    </>
                ) : (
                    <>
                        <div className="explorer-section">
                            <span className="material-symbols-outlined section-chevron">chevron_right</span>
                            <span className="section-title">SRC</span>
                        </div>
                        <div className="explorer-controls">
                            <input
                                className="ui-input mono"
                                placeholder="absolute project root path"
                                value={inputRoot}
                                onChange={(e) => setInputRoot(e.currentTarget.value)}
                            />
                            <div className="explorer-controls-row">
                                <button type="button" className="ui-button" onClick={onUseIsolatedRoot}>
                                    Isolated
                                </button>
                                <button type="button" className="ui-button is-primary" onClick={onOpenProject}>
                                    Open
                                </button>
                                <button
                                    type="button"
                                    className="ui-button"
                                    onClick={onRefreshFiles}
                                    disabled={!project}
                                >
                                    Refresh
                                </button>
                            </div>
                            <div className="explorer-controls-row">
                                <input
                                    className="ui-input"
                                    placeholder="new file name"
                                    value={newFileName}
                                    onChange={(e) => setNewFileName(e.currentTarget.value)}
                                    disabled={!project}
                                />
                                <button
                                    type="button"
                                    className="ui-button"
                                    onClick={onCreateFile}
                                    disabled={!project || !newFileName.trim()}
                                >
                                    +
                                </button>
                            </div>
                        </div>
                        {project && <div className="explorer-note mono">{project.root}</div>}
                        <div className="explorer-scroll">
                            {sortedFiles.length === 0 && (
                                <div className="tree-row is-disabled">No files in root</div>
                            )}
                            {sortedFiles.map((f) => {
                                const isActive = selectedPath === f.path;
                                const rowClassName =
                                    `tree-row${isActive ? " is-active" : ""}${f.is_dir ? " is-disabled" : ""}`;
                                const iconName = iconNameForPath(f.path, f.is_dir);
                                const iconTone = iconToneClass(f.path, f.is_dir);

                                return (
                                    <div
                                        key={`${f.path}-${f.is_dir ? "d" : "f"}`}
                                        className={rowClassName}
                                        onClick={() => {
                                            if (!f.is_dir) {
                                                void onRead(f.path);
                                            }
                                        }}
                                    >
                                        <span className="material-symbols-outlined tree-chevron">
                                            {f.is_dir ? "chevron_right" : ""}
                                        </span>
                                        <span className={`material-symbols-outlined tree-icon ${iconTone}`}>
                                            {iconName}
                                        </span>
                                        <span className="tree-label">{basename(f.path)}</span>
                                        {isActive && isDirty && <span className="tree-modified">M</span>}
                                    </div>
                                );
                            })}
                        </div>
                    </>
                )}
            </aside>

            <section className="workspace-panel">
                <div className="tab-strip">
                    {openTabs.length === 0 && (
                        <div className="tab is-active">
                            <span className="material-symbols-outlined tab-icon tone-story">article</span>
                            <span className="tab-title">untitled.story</span>
                        </div>
                    )}
                    {openTabs.map((tabPath) => (
                        <button
                            key={tabPath}
                            type="button"
                            className={`tab ${selectedPath === tabPath ? "is-active" : ""}`}
                            onClick={() => {
                                if (selectedPath !== tabPath) {
                                    void onRead(tabPath);
                                }
                            }}
                        >
                            <span
                                className={`material-symbols-outlined tab-icon ${iconToneClass(tabPath, false)}`}
                            >
                                {iconNameForPath(tabPath, false)}
                            </span>
                            <span className="tab-title">{basename(tabPath)}</span>
                            <span
                                className="tab-close"
                                onClick={(e) => {
                                    e.stopPropagation();
                                    onCloseTab(tabPath);
                                }}
                            >
                                <span className="material-symbols-outlined tab-close-icon">close</span>
                            </span>
                        </button>
                    ))}
                </div>

                <div className="breadcrumb-bar">
                    {breadcrumbs.length === 0 && <span>No file selected</span>}
                    {breadcrumbs.map((part, idx) => (
                        <span key={`${part}-${idx}`}>
                            {idx > 0 && (
                                <span className="material-symbols-outlined breadcrumb-icon">chevron_right</span>
                            )}
                            <span>{part}</span>
                        </span>
                    ))}
                    {isDirty && <span className="breadcrumb-sep">*</span>}
                </div>

                <div className="editor-main">
                    <div className="gutter">
                        {lineNumbers.map((n) => (
                            <div
                                key={`line-${n}`}
                                className={`gutter-line ${n === cursor.line ? "is-active" : ""}`}
                            >
                                {n}
                            </div>
                        ))}
                    </div>

                    <div className="editor-scroll">
                        {!selectedPath && (
                            <div className="empty-state">
                                <div className="material-symbols-outlined empty-icon">article</div>
                                <div>Open a file to start writing.</div>
                            </div>
                        )}
                        {selectedPath && (
                            <textarea
                                className="ui-textarea mono editor-textarea"
                                value={content}
                                onChange={(e) =>
                                    onEditorChange(
                                        e.currentTarget.value,
                                        e.currentTarget.selectionStart ?? 0,
                                    )
                                }
                                onKeyUp={(e) =>
                                    onEditorSelect(
                                        e.currentTarget.value,
                                        e.currentTarget.selectionStart ?? 0,
                                    )
                                }
                                onClick={(e) =>
                                    onEditorSelect(
                                        e.currentTarget.value,
                                        e.currentTarget.selectionStart ?? 0,
                                    )
                                }
                                onSelect={(e) =>
                                    onEditorSelect(
                                        e.currentTarget.value,
                                        e.currentTarget.selectionStart ?? 0,
                                    )
                                }
                                spellCheck={false}
                            />
                        )}
                    </div>

                    <div className="minimap">
                        <div className="minimap-inner">
                            {minimapRows.map((w, idx) => (
                                <div
                                    key={`mini-${idx}`}
                                    className="minimap-line"
                                    style={{ width: `${w}%` }}
                                />
                            ))}
                            <div className="minimap-viewport" style={{ top: `${minimapViewportTop}%` }} />
                        </div>
                    </div>
                </div>
            </section>

            <footer className="status-bar">
                <div className="status-group">
                    <button type="button" className="status-item status-button">
                        <span className="material-symbols-outlined status-icon">alt_route</span>
                        <span>{`main@${headShort}${isDirty ? "*" : ""}`}</span>
                    </button>
                    <button type="button" className="status-item status-button" onClick={onRefreshFiles}>
                        <span className="material-symbols-outlined status-icon">sync</span>
                    </button>
                    <button
                        type="button"
                        className="status-item status-button"
                        onClick={onSave}
                        disabled={!selectedPath}
                    >
                        Save
                    </button>
                    <span className="status-item">
                        <span className="material-symbols-outlined status-icon">cancel</span>
                        <span>0</span>
                    </span>
                    <span className="status-item">
                        <span className="material-symbols-outlined status-icon">warning</span>
                        <span>0</span>
                    </span>
                    <span className="status-item">{`#${vcsState?.node_count ?? 0}`}</span>
                </div>

                <div className="status-group">
                    <span className="status-item">{`Ln ${cursor.line}, Col ${cursor.col}`}</span>
                    <span className="status-item">UTF-8</span>
                    <span className="status-item">
                        <span className="material-symbols-outlined status-icon">code</span>
                        <span>{language}</span>
                    </span>
                    <span className="status-item">
                        <span className="material-symbols-outlined status-icon">notifications</span>
                    </span>
                </div>
            </footer>

            {!!error && (
                <div className="status-toast" role="alert">
                    {error}
                </div>
            )}
        </main>
    );
}
