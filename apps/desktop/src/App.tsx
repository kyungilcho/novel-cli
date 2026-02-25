import { useEffect, useMemo, useState } from "react";
import { ActivityBar, type ActivityView } from "./components/layout/ActivityBar";
import { EditorPane } from "./components/layout/EditorPane";
import { ExplorerPanel } from "./components/layout/ExplorerPanel";
import { HistoryPanel } from "./components/layout/HistoryPanel";
import { StatusBar } from "./components/layout/StatusBar";
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

type CursorPosition = {
    line: number;
    col: number;
};

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
            <ActivityBar activeView={activeView} onSelectActivity={onSelectActivity} />

            <aside className="explorer-panel">
                <div className="explorer-header">
                    {activeView === "history" ? "Source Control" : "Explorer"}
                </div>

                {activeView === "history" ? (
                    <HistoryPanel
                        vcsState={vcsState}
                        logNodes={logNodes}
                        commitMessage={commitMessage}
                        onCommitMessageChange={setCommitMessage}
                        onCommitSnapshot={onCommitSnapshot}
                        onRefreshVcs={onRefreshVcs}
                        onCheckoutSnapshot={onCheckoutSnapshot}
                        hasProject={project !== null}
                        vcsBusy={vcsBusy}
                    />
                ) : (
                    <ExplorerPanel
                        inputRoot={inputRoot}
                        onInputRootChange={setInputRoot}
                        newFileName={newFileName}
                        onNewFileNameChange={setNewFileName}
                        hasProject={project !== null}
                        projectRoot={project?.root ?? null}
                        selectedPath={selectedPath}
                        isDirty={isDirty}
                        sortedFiles={sortedFiles}
                        onUseIsolatedRoot={onUseIsolatedRoot}
                        onOpenProject={onOpenProject}
                        onRefreshFiles={onRefreshFiles}
                        onCreateFile={onCreateFile}
                        onRead={onRead}
                    />
                )}
            </aside>

            <EditorPane
                openTabs={openTabs}
                selectedPath={selectedPath}
                content={content}
                breadcrumbs={breadcrumbs}
                isDirty={isDirty}
                cursor={cursor}
                lineNumbers={lineNumbers}
                minimapRows={minimapRows}
                minimapViewportTop={minimapViewportTop}
                onRead={onRead}
                onCloseTab={onCloseTab}
                onEditorChange={onEditorChange}
                onEditorSelect={onEditorSelect}
            />

            <StatusBar
                headShort={headShort}
                isDirty={isDirty}
                nodeCount={vcsState?.node_count ?? 0}
                cursor={cursor}
                language={language}
                canSave={selectedPath !== ""}
                onRefreshFiles={onRefreshFiles}
                onSave={onSave}
            />

            {!!error && (
                <div className="status-toast" role="alert">
                    {error}
                </div>
            )}
        </main>
    );
}
