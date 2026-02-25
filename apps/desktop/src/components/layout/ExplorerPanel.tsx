import type { FileEntry } from "../../lib/api/fileApi";
import { Button, Input } from "../ui";

type ExplorerPanelProps = {
    inputRoot: string;
    onInputRootChange: (value: string) => void;
    newFileName: string;
    onNewFileNameChange: (value: string) => void;
    hasProject: boolean;
    projectRoot: string | null;
    selectedPath: string;
    isDirty: boolean;
    sortedFiles: FileEntry[];
    onUseIsolatedRoot: () => void | Promise<void>;
    onOpenProject: () => void | Promise<void>;
    onRefreshFiles: () => void | Promise<void>;
    onCreateFile: () => void | Promise<void>;
    onRead: (path: string) => void | Promise<void>;
};

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

export function ExplorerPanel({
    inputRoot,
    onInputRootChange,
    newFileName,
    onNewFileNameChange,
    hasProject,
    projectRoot,
    selectedPath,
    isDirty,
    sortedFiles,
    onUseIsolatedRoot,
    onOpenProject,
    onRefreshFiles,
    onCreateFile,
    onRead,
}: ExplorerPanelProps) {
    return (
        <>
            <div className="explorer-section">
                <span className="material-symbols-outlined section-chevron">chevron_right</span>
                <span className="section-title">SRC</span>
            </div>
            <div className="explorer-controls">
                <Input
                    className="mono"
                    placeholder="absolute project root path"
                    value={inputRoot}
                    onChange={(e) => onInputRootChange(e.currentTarget.value)}
                />
                <div className="explorer-controls-row">
                    <Button onClick={() => void onUseIsolatedRoot()}>Isolated</Button>
                    <Button onClick={() => void onOpenProject()} variant="primary">
                        Open
                    </Button>
                    <Button onClick={() => void onRefreshFiles()} disabled={!hasProject}>
                        Refresh
                    </Button>
                </div>
                <div className="explorer-controls-row">
                    <Input
                        placeholder="new file name"
                        value={newFileName}
                        onChange={(e) => onNewFileNameChange(e.currentTarget.value)}
                        disabled={!hasProject}
                    />
                    <Button onClick={() => void onCreateFile()} disabled={!hasProject || !newFileName.trim()}>
                        +
                    </Button>
                </div>
            </div>
            {projectRoot && <div className="explorer-note mono">{projectRoot}</div>}
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
    );
}
