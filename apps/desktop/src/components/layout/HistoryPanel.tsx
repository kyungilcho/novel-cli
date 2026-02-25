import { Button, Textarea } from "../ui";
import type { RepoState, VersionNode } from "../../lib/api/vcsApi";

type HistoryPanelProps = {
    vcsState: RepoState | null;
    logNodes: VersionNode[];
    commitMessage: string;
    onCommitMessageChange: (value: string) => void;
    onCommitSnapshot: () => void | Promise<void>;
    onRefreshVcs: () => void | Promise<void>;
    onCheckoutSnapshot: (nodeId: string) => void | Promise<void>;
    hasProject: boolean;
    vcsBusy: boolean;
};

function formatNodeTime(unixMs: number): string {
    return new Date(unixMs).toLocaleString();
}

export function HistoryPanel({
    vcsState,
    logNodes,
    commitMessage,
    onCommitMessageChange,
    onCommitSnapshot,
    onRefreshVcs,
    onCheckoutSnapshot,
    hasProject,
    vcsBusy,
}: HistoryPanelProps) {
    return (
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
                <Textarea
                    className="history-message"
                    placeholder="snapshot message..."
                    value={commitMessage}
                    onChange={(e) => onCommitMessageChange(e.currentTarget.value)}
                    disabled={!hasProject || vcsBusy}
                    rows={3}
                />
                <div className="explorer-controls-row">
                    <Button
                        onClick={() => void onCommitSnapshot()}
                        disabled={!hasProject || vcsBusy || !commitMessage.trim()}
                        variant="primary"
                    >
                        Commit
                    </Button>
                    <Button onClick={() => void onRefreshVcs()} disabled={!hasProject || vcsBusy}>
                        Refresh Log
                    </Button>
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
                                <Button
                                    className="history-item-btn"
                                    onClick={() => void onCheckoutSnapshot(node.id)}
                                    disabled={!hasProject || vcsBusy || isHead}
                                >
                                    {isHead ? "Current" : "Checkout"}
                                </Button>
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
    );
}
