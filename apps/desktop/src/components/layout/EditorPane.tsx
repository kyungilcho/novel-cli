import { Button, Textarea } from "../ui";

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
    onRead,
    onCloseTab,
    onEditorChange,
    onEditorSelect,
}: EditorPaneProps) {
    return (
        <section className="workspace-panel">
            <div className="tab-strip">
                {openTabs.length === 0 && (
                    <div className="tab is-active">
                        <span className="material-symbols-outlined tab-icon tone-story">article</span>
                        <span className="tab-title">untitled.story</span>
                    </div>
                )}
                {openTabs.map((tabPath) => (
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
                {breadcrumbs.length === 0 && <span>No file selected</span>}
                {breadcrumbs.map((part, idx) => (
                    <span key={`${part}-${idx}`}>
                        {idx > 0 && <span className="material-symbols-outlined breadcrumb-icon">chevron_right</span>}
                        <span>{part}</span>
                    </span>
                ))}
                {isDirty && <span className="breadcrumb-sep">*</span>}
            </div>

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
        </section>
    );
}
