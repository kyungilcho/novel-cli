import { Button, Textarea } from "../ui";
import type { FileDiff, NodeDiff } from "../../lib/api/vcsApi";

type CursorPosition = {
    line: number;
    col: number;
};

type EditorPaneProps = {
    openTabs: string[];
    selectedPath: string;
    content: string;
    breadcrumbs: string[];
    isDirty: boolean;
    cursor: CursorPosition;
    lineNumbers: number[];
    minimapRows: number[];
    minimapViewportTop: number;
    diffResult: NodeDiff | null;
    selectedDiffPath: string;
    onExitDiff: () => void;
    onRead: (path: string) => void | Promise<void>;
    onCloseTab: (path: string) => void;
    onEditorChange: (text: string, selectionStart: number) => void;
    onEditorSelect: (text: string, selectionStart: number) => void;
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

function buildChangeOnlyDiffText(fileDiff: FileDiff): string {
    if (fileDiff.is_binary) return "";

    if (fileDiff.unified) {
        const changedLines = fileDiff.unified
            .split("\n")
            .filter((line) => {
                if (line.startsWith("\\ No newline at end of file")) return false;
                if (line.startsWith("@@")) return false;
                if (line.startsWith("---")) return false;
                if (line.startsWith("+++")) return false;
                return line.startsWith("+") || line.startsWith("-");
            });

        if (changedLines.length > 0) {
            return changedLines.join("\n");
        }
    }

    if (fileDiff.kind === "added" && fileDiff.after_text) {
        return fileDiff.after_text.split("\n").map((line) => `+${line}`).join("\n");
    }
    if (fileDiff.kind === "removed" && fileDiff.before_text) {
        return fileDiff.before_text.split("\n").map((line) => `-${line}`).join("\n");
    }

    return "";
}

export function EditorPane({
    openTabs,
    selectedPath,
    content,
    breadcrumbs,
    isDirty,
    cursor,
    lineNumbers,
    minimapRows,
    minimapViewportTop,
    diffResult,
    selectedDiffPath,
    onExitDiff,
    onRead,
    onCloseTab,
    onEditorChange,
    onEditorSelect,
}: EditorPaneProps) {
    const activeDiff: FileDiff | null = diffResult
        ? diffResult.files.find((f) => f.path === selectedDiffPath) ?? diffResult.files[0] ?? null
        : null;
    const activeDiffText = activeDiff ? buildChangeOnlyDiffText(activeDiff) : "";
    const activeDiffLines = activeDiffText.length > 0 ? activeDiffText.split("\n") : [];

    return (
        <section className="workspace-panel">
            <div className="tab-strip">
                {diffResult && (
                    <div className="tab is-active diff-tab">
                        <span className="material-symbols-outlined tab-icon tone-story">difference</span>
                        <span className="tab-title">{`Diff ${diffResult.from.slice(0, 8)} -> ${diffResult.to.slice(0, 8)}`}</span>
                        <span className="tab-close" onClick={onExitDiff}>
                            <span className="material-symbols-outlined tab-close-icon">close</span>
                        </span>
                    </div>
                )}
                {!diffResult && openTabs.length === 0 && (
                    <div className="tab is-active">
                        <span className="material-symbols-outlined tab-icon tone-story">article</span>
                        <span className="tab-title">untitled.story</span>
                    </div>
                )}
                {!diffResult && openTabs.map((tabPath) => (
                    <Button
                        key={tabPath}
                        className={`tab ${selectedPath === tabPath ? "is-active" : ""}`}
                        unstyled
                        onClick={() => {
                            if (selectedPath !== tabPath) {
                                void onRead(tabPath);
                            }
                        }}
                    >
                        <span className={`material-symbols-outlined tab-icon ${iconToneClass(tabPath, false)}`}>
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
                    </Button>
                ))}
            </div>

            <div className="breadcrumb-bar">
                {diffResult ? (
                    <>
                        <span>Diff View</span>
                        {activeDiff && (
                            <>
                                <span className="material-symbols-outlined breadcrumb-icon">chevron_right</span>
                                <span>{activeDiff.path}</span>
                            </>
                        )}
                    </>
                ) : (
                    <>
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
                    </>
                )}
            </div>

            {diffResult ? (
                <div className="diff-main">
                    <div className="diff-detail">
                        {!activeDiff && <div className="empty-state">Select a changed file.</div>}
                        {activeDiff && activeDiff.is_binary && (
                            <div className="empty-state">Binary file changed.</div>
                        )}
                        {activeDiff && !activeDiff.is_binary && (
                            <>
                                {activeDiffLines.length === 0 && (
                                    <pre className="diff-pre mono">(no textual diff)</pre>
                                )}
                                {activeDiffLines.length > 0 && (
                                    <div className="diff-lines mono">
                                        {activeDiffLines.map((line, idx) => {
                                            const sign = line.startsWith("+")
                                                ? "+"
                                                : line.startsWith("-")
                                                  ? "-"
                                                  : " ";
                                            const tone = sign === "+" ? "is-added" : sign === "-" ? "is-removed" : "";
                                            const content = sign === " " ? line : line.slice(1);
                                            return (
                                                <div
                                                    key={`diff-line-${idx}`}
                                                    className={`diff-line ${tone}`.trim()}
                                                >
                                                    <span className={`diff-line-sign ${tone}`.trim()}>{sign}</span>
                                                    <span className="diff-line-content">
                                                        {content.length > 0 ? content : "\u00a0"}
                                                    </span>
                                                </div>
                                            );
                                        })}
                                    </div>
                                )}
                            </>
                        )}
                    </div>
                </div>
            ) : (
                <div className="editor-main">
                <div className="gutter">
                    {lineNumbers.map((n) => (
                        <div key={`line-${n}`} className={`gutter-line ${n === cursor.line ? "is-active" : ""}`}>
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
                        <Textarea
                            className="mono editor-textarea"
                            value={content}
                            onChange={(e) =>
                                onEditorChange(e.currentTarget.value, e.currentTarget.selectionStart ?? 0)
                            }
                            onKeyUp={(e) =>
                                onEditorSelect(e.currentTarget.value, e.currentTarget.selectionStart ?? 0)
                            }
                            onClick={(e) =>
                                onEditorSelect(e.currentTarget.value, e.currentTarget.selectionStart ?? 0)
                            }
                            onSelect={(e) =>
                                onEditorSelect(e.currentTarget.value, e.currentTarget.selectionStart ?? 0)
                            }
                            spellCheck={false}
                        />
                    )}
                </div>

                <div className="minimap">
                    <div className="minimap-inner">
                        {minimapRows.map((w, idx) => (
                            <div key={`mini-${idx}`} className="minimap-line" style={{ width: `${w}%` }} />
                        ))}
                        <div className="minimap-viewport" style={{ top: `${minimapViewportTop}%` }} />
                    </div>
                </div>
                </div>
            )}
        </section>
    );
}
