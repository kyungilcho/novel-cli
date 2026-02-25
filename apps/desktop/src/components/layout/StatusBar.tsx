import { Button } from "../ui";

type CursorPosition = {
    line: number;
    col: number;
};

type StatusBarProps = {
    headShort: string;
    isDirty: boolean;
    nodeCount: number;
    cursor: CursorPosition;
    language: string;
    canSave: boolean;
    onRefreshFiles: () => void | Promise<void>;
    onSave: () => void | Promise<void>;
};

export function StatusBar({
    headShort,
    isDirty,
    nodeCount,
    cursor,
    language,
    canSave,
    onRefreshFiles,
    onSave,
}: StatusBarProps) {
    return (
        <footer className="status-bar">
            <div className="status-group">
                <Button className="status-item status-button" unstyled>
                    <span className="material-symbols-outlined status-icon">alt_route</span>
                    <span>{`main@${headShort}${isDirty ? "*" : ""}`}</span>
                </Button>
                <Button className="status-item status-button" unstyled onClick={() => void onRefreshFiles()}>
                    <span className="material-symbols-outlined status-icon">sync</span>
                </Button>
                <Button className="status-item status-button" unstyled onClick={() => void onSave()} disabled={!canSave}>
                    Save
                </Button>
                <span className="status-item">
                    <span className="material-symbols-outlined status-icon">cancel</span>
                    <span>0</span>
                </span>
                <span className="status-item">
                    <span className="material-symbols-outlined status-icon">warning</span>
                    <span>0</span>
                </span>
                <span className="status-item">{`#${nodeCount}`}</span>
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
    );
}
