import { useEffect, useState } from "react";
import { defaultWorkspaceRoot, openProject } from "./lib/api/projectApi";
import { createFile, listFiles, readFile, writeFile } from "./lib/api/fileApi";
import type { ProjectInfo } from "./lib/api/projectApi";
import type { FileEntry } from "./lib/api/fileApi";

export default function App() {
    const [inputRoot, setInputRoot] = useState("");
    const [project, setProject] = useState<ProjectInfo | null>(null);
    const [files, setFiles] = useState<FileEntry[]>([]);
    const [selectedPath, setSelectedPath] = useState("");
    const [content, setContent] = useState("");
    const [error, setError] = useState("");
    const [newFileName, setNewFileName] = useState("");

    useEffect(() => {
        let cancelled = false;

        const initDefaultRoot = async () => {
            try {
                const root = await defaultWorkspaceRoot();
                if (!cancelled) {
                    setInputRoot(root);
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
        } catch (e) {
            setError(String(e));
        }
    };

    const onOpenProject = async () => {
        setError("");
        try {
            const p = await openProject(inputRoot.trim());
            setProject(p);
            setSelectedPath("");
            setContent("");

            const list = await listFiles(p.root, ".");
            setFiles(list);
        } catch (e) {
            setError(String(e));
        }
    };

    const onRefreshFiles = async () => {
        if (!project) return;
        setError("");
        try {
            const list = await listFiles(project.root, ".");
            setFiles(list);
        } catch (e) {
            setError(String(e));
        }
    };

    const onRead = async (path: string) => {
        if (!project) return;
        if (path.endsWith("/")) return;

        setError("");
        try {
            const text = await readFile(project.root, path);
            setSelectedPath(path);
            setContent(text);
        } catch (e) {
            setError(String(e));
        }
    };

    const onSave = async () => {
        if (!project || !selectedPath) return;
        setError("");
        try {
            await writeFile(project.root, selectedPath, content);
        } catch (e) {
            setError(String(e));
        }
    };

    const onCreateFile = async () => {
        if (!project || !newFileName.trim()) return;
        setError("");
        try {
            await createFile(project.root, ".", newFileName.trim());
            setNewFileName("");
            await onRefreshFiles();
        } catch (e) {
            setError(String(e));
        }
    };

    return (
        <main style={{ maxWidth: 900, margin: "24px auto", padding: 16 }}>
            <h1>Novel IDE (Scaffold)</h1>

            <div style={{ display: "flex", gap: 8, marginBottom: 12 }}>
                <input
                    style={{ flex: 1 }}
                    placeholder="project root path (absolute)"
                    value={inputRoot}
                    onChange={(e) => setInputRoot(e.currentTarget.value)}
                />
                <button type="button" onClick={onUseIsolatedRoot}>
                    Isolated Root
                </button>
                <button type="button" onClick={onOpenProject}>
                    Open
                </button>
                <button type="button" onClick={onRefreshFiles} disabled={!project}>
                    Refresh Files
                </button>
            </div>

            {project && (
                <p>
                    Opened: <b>{project.name}</b> ({project.root})
                </p>
            )}

            {error && <p style={{ color: "crimson" }}>{error}</p>}

            <div
                style={{
                    display: "grid",
                    gridTemplateColumns: "280px 1fr",
                    gap: 12,
                }}
            >
                <section style={{ border: "1px solid #ddd", padding: 12 }}>
                    <h3>Files</h3>
                    <div style={{ display: "flex", gap: 4, marginBottom: 8 }}>
                        <input
                            placeholder="New file name..."
                            value={newFileName}
                            onChange={(e) => setNewFileName(e.currentTarget.value)}
                            disabled={!project}
                            style={{ flex: 1, minWidth: 0 }}
                        />
                        <button
                            type="button"
                            onClick={onCreateFile}
                            disabled={!project || !newFileName.trim()}
                        >
                            +
                        </button>
                    </div>
                    <ul style={{ paddingLeft: 16 }}>
                        {files.map((f) => {
                            const label = f.is_dir ? `${f.path}/` : f.path;
                            return (
                                <li key={`${f.path}-${f.is_dir ? "d" : "f"}`}>
                                    <button
                                        type="button"
                                        onClick={() => onRead(f.path)}
                                        disabled={f.is_dir}
                                        style={{
                                            background: "transparent",
                                            border: "none",
                                            cursor: f.is_dir ? "default" : "pointer",
                                        }}
                                    >
                                        {label}
                                    </button>
                                </li>
                            );
                        })}
                    </ul>
                </section>

                <section style={{ border: "1px solid #ddd", padding: 12 }}>
                    <h3>Editor</h3>
                    <p>Selected: {selectedPath || "-"}</p>
                    <textarea
                        style={{ width: "100%", minHeight: 360 }}
                        value={content}
                        onChange={(e) => setContent(e.currentTarget.value)}
                        disabled={!selectedPath}
                    />
                    <div style={{ marginTop: 8 }}>
                        <button type="button" onClick={onSave} disabled={!selectedPath}>
                            Save
                        </button>
                    </div>
                </section>
            </div>
        </main>
    );
}
