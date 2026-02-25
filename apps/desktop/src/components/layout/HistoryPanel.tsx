import { useEffect, useMemo, useRef, useState } from "react";

import { Button, Textarea } from "../ui";
import type { NodeDiff, RepoState, VersionNode } from "../../lib/api/vcsApi";

type HistoryPanelProps = {
    vcsState: RepoState | null;
    logNodes: VersionNode[];
    commitMessage: string;
    expandedNodeId: string;
    expandedDiff: NodeDiff | null;
    loadingNodeId: string;
    selectedDiffPath: string;
    activeDiffToNodeId: string;
    onCommitMessageChange: (value: string) => void;
    onToggleNodeExpand: (node: VersionNode) => void | Promise<void>;
    onSelectExpandedDiffFile: (nodeId: string, path: string) => void;
    onCommitSnapshot: () => void | Promise<void>;
    onRefreshVcs: () => void | Promise<void>;
    onCheckoutSnapshot: (nodeId: string) => void | Promise<void>;
    hasProject: boolean;
    vcsBusy: boolean;
};

type HistoryContextMenu = {
    nodeId: string;
    x: number;
    y: number;
};

const GRAPH_LANE_GAP_PX = 12;
const GRAPH_MIN_WIDTH_PX = 40;
const GRAPH_WIDTH_PADDING_PX = 8;
const GRAPH_ROW_HEIGHT_PX = 34;
const GRAPH_NODE_Y_PX = 17;
const GRAPH_NODE_RADIUS_PX = 6;
const GRAPH_LANE_X_OFFSET_PX = 10;

const EXPAND_EMPTY_HEIGHT_PX = 30;
const EXPAND_LIST_ROW_HEIGHT_PX = 28;
const EXPAND_LIST_VERTICAL_PADDING_PX = 4;
const EXPAND_LIST_MAX_HEIGHT_PX = 180;

const CONTEXT_MENU_WIDTH_PX = 180;
const CONTEXT_MENU_HEIGHT_PX = 40;
const CONTEXT_MENU_VIEWPORT_MARGIN_PX = 8;

const COMMIT_ID_SHORT_LEN = 8;

function formatNodeTime(unixMs: number): string {
    return new Date(unixMs).toLocaleString();
}

type GraphRow = {
    node: VersionNode;
    lane: number;
    parentLanes: number[];
    laneCount: number;
};

function buildGraphRows(nodes: VersionNode[]): {
    rows: GraphRow[];
    maxLanes: number;
} {
    const rows: GraphRow[] = [];
    let laneHeads: string[] = [];
    let maxLanes = 1;

    for (const node of nodes) {
        let lane = laneHeads.indexOf(node.id);
        if (lane === -1) {
            laneHeads.push(node.id);
            lane = laneHeads.length - 1;
        }

        const beforeCount = laneHeads.length;
        let nextLaneHeads = [...laneHeads];

        if (node.parents.length > 0) {
            nextLaneHeads[lane] = node.parents[0];
        } else {
            nextLaneHeads.splice(lane, 1);
        }

        for (const parentId of node.parents.slice(1)) {
            if (!nextLaneHeads.includes(parentId)) {
                const insertAt = Math.min(lane + 1, nextLaneHeads.length);
                nextLaneHeads.splice(insertAt, 0, parentId);
            }
        }

        nextLaneHeads = nextLaneHeads.filter(
            (id, idx) => nextLaneHeads.indexOf(id) === idx,
        );

        const parentLanes = node.parents
            .map((parentId) => nextLaneHeads.indexOf(parentId))
            .filter((idx) => idx >= 0);

        rows.push({
            node,
            lane,
            parentLanes,
            laneCount: Math.max(1, beforeCount, nextLaneHeads.length),
        });

        maxLanes = Math.max(
            maxLanes,
            beforeCount,
            nextLaneHeads.length,
            lane + 1,
        );
        for (const parentLane of parentLanes) {
            maxLanes = Math.max(maxLanes, parentLane + 1);
        }

        laneHeads = nextLaneHeads;
    }

    return { rows, maxLanes };
}

export function HistoryPanel({
    vcsState,
    logNodes,
    commitMessage,
    expandedNodeId,
    expandedDiff,
    loadingNodeId,
    selectedDiffPath,
    activeDiffToNodeId,
    onCommitMessageChange,
    onToggleNodeExpand,
    onSelectExpandedDiffFile,
    onCommitSnapshot,
    onRefreshVcs,
    onCheckoutSnapshot,
    hasProject,
    vcsBusy,
}: HistoryPanelProps) {
    const { rows: graphRows, maxLanes } = useMemo(
        () => buildGraphRows(logNodes),
        [logNodes],
    );
    const [contextMenu, setContextMenu] = useState<HistoryContextMenu | null>(
        null,
    );
    const contextMenuRef = useRef<HTMLDivElement | null>(null);

    const graphWidth = Math.max(
        GRAPH_MIN_WIDTH_PX,
        maxLanes * GRAPH_LANE_GAP_PX + GRAPH_WIDTH_PADDING_PX,
    );
    const yTop = 0;
    const yNode = GRAPH_NODE_Y_PX;

    useEffect(() => {
        if (!contextMenu) return;

        const onPointerDown = (event: PointerEvent) => {
            const target = event.target as Node | null;
            if (target && contextMenuRef.current?.contains(target)) return;
            setContextMenu(null);
        };

        const onKeyDown = (event: KeyboardEvent) => {
            if (event.key === "Escape") {
                setContextMenu(null);
            }
        };

        window.addEventListener("pointerdown", onPointerDown);
        window.addEventListener("keydown", onKeyDown);
        return () => {
            window.removeEventListener("pointerdown", onPointerDown);
            window.removeEventListener("keydown", onKeyDown);
        };
    }, [contextMenu]);

    const contextNode = contextMenu
        ? (logNodes.find((n) => n.id === contextMenu.nodeId) ?? null)
        : null;
    const contextNodeIsHead = contextNode
        ? vcsState?.head === contextNode.id
        : false;
    const canContextCheckout = Boolean(
        contextNode && hasProject && !vcsBusy && !contextNodeIsHead,
    );

    return (
        <>
            <div className="explorer-section">
                <span className="material-symbols-outlined section-chevron">
                    chevron_right
                </span>
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
                    onChange={(e) =>
                        onCommitMessageChange(e.currentTarget.value)
                    }
                    disabled={!hasProject || vcsBusy}
                    rows={3}
                />
                <div className="explorer-controls-row">
                    <Button
                        onClick={() => void onCommitSnapshot()}
                        disabled={
                            !hasProject || vcsBusy || !commitMessage.trim()
                        }
                        variant="primary"
                    >
                        Commit
                    </Button>
                    <Button
                        onClick={() => void onRefreshVcs()}
                        disabled={!hasProject || vcsBusy}
                    >
                        Refresh Log
                    </Button>
                </div>
            </div>
            <div className="explorer-scroll">
                {graphRows.length === 0 && (
                    <div className="tree-row is-disabled">No snapshots yet</div>
                )}
                {graphRows.map((row) => {
                    const node = row.node;
                    const isHead = vcsState?.head === node.id;
                    const isExpanded = expandedNodeId === node.id;
                    const isLoading = loadingNodeId === node.id;
                    const hasPrevious = node.parents.length > 0;
                    const files =
                        isExpanded && expandedDiff ? expandedDiff.files : [];

                    let expandPanelHeight = 0;
                    if (isExpanded) {
                        if (!hasPrevious || isLoading || files.length === 0) {
                            expandPanelHeight = EXPAND_EMPTY_HEIGHT_PX;
                        } else {
                            expandPanelHeight = Math.min(
                                EXPAND_LIST_MAX_HEIGHT_PX,
                                files.length * EXPAND_LIST_ROW_HEIGHT_PX +
                                    EXPAND_LIST_VERTICAL_PADDING_PX,
                            );
                        }
                    }

                    const totalHeight = GRAPH_ROW_HEIGHT_PX + expandPanelHeight;

                    return (
                        <div
                            key={node.id}
                            className={`history-graph-row${isHead ? " is-head" : ""}${isExpanded ? " is-expanded" : ""}`}
                            role="button"
                            tabIndex={0}
                            onClick={() => void onToggleNodeExpand(node)}
                            onContextMenu={(e) => {
                                e.preventDefault();
                                const x = Math.max(
                                    CONTEXT_MENU_VIEWPORT_MARGIN_PX,
                                    Math.min(
                                        e.clientX,
                                        window.innerWidth -
                                            CONTEXT_MENU_WIDTH_PX -
                                            CONTEXT_MENU_VIEWPORT_MARGIN_PX,
                                    ),
                                );
                                const y = Math.max(
                                    CONTEXT_MENU_VIEWPORT_MARGIN_PX,
                                    Math.min(
                                        e.clientY,
                                        window.innerHeight -
                                            CONTEXT_MENU_HEIGHT_PX -
                                            CONTEXT_MENU_VIEWPORT_MARGIN_PX,
                                    ),
                                );
                                setContextMenu({ nodeId: node.id, x, y });
                            }}
                            onKeyDown={(e) => {
                                if (e.key === "Enter" || e.key === " ") {
                                    e.preventDefault();
                                    void onToggleNodeExpand(node);
                                }
                            }}
                        >
                            <div
                                className="history-graph-lane"
                                style={{
                                    width: graphWidth,
                                    height: totalHeight,
                                }}
                            >
                                <svg
                                    width={graphWidth}
                                    height={totalHeight}
                                    viewBox={`0 0 ${graphWidth} ${totalHeight}`}
                                >
                                    {Array.from(
                                        {
                                            length: Math.max(
                                                maxLanes,
                                                row.laneCount,
                                            ),
                                        },
                                        (_, idx) => {
                                            const x =
                                                idx * GRAPH_LANE_GAP_PX +
                                                GRAPH_LANE_X_OFFSET_PX;
                                            return (
                                                <line
                                                    key={`lane-${node.id}-${idx}`}
                                                    className="history-graph-line"
                                                    x1={x}
                                                    y1={yTop}
                                                    x2={x}
                                                    y2={totalHeight}
                                                />
                                            );
                                        },
                                    )}
                                    {row.parentLanes.map((parentLane, idx) => {
                                        const x1 =
                                            row.lane * GRAPH_LANE_GAP_PX +
                                            GRAPH_LANE_X_OFFSET_PX;
                                        const x2 =
                                            parentLane * GRAPH_LANE_GAP_PX +
                                            GRAPH_LANE_X_OFFSET_PX;
                                        return (
                                            <line
                                                key={`branch-${node.id}-${idx}`}
                                                className="history-graph-branch"
                                                x1={x1}
                                                y1={yNode}
                                                x2={x2}
                                                y2={GRAPH_ROW_HEIGHT_PX}
                                            />
                                        );
                                    })}
                                    <circle
                                        className={[
                                            "history-graph-node",
                                            isHead ? "is-head" : "",
                                        ]
                                            .filter(Boolean)
                                            .join(" ")}
                                        cx={
                                            row.lane * GRAPH_LANE_GAP_PX +
                                            GRAPH_LANE_X_OFFSET_PX
                                        }
                                        cy={yNode}
                                        r={GRAPH_NODE_RADIUS_PX}
                                    />
                                </svg>
                            </div>
                            <div className="history-graph-content">
                                <div className="history-item-top">
                                    <div className="history-item-header">
                                        <span className="material-symbols-outlined history-open-icon">
                                            {isExpanded
                                                ? "expand_more"
                                                : "chevron_right"}
                                        </span>
                                        <span className="history-item-message">
                                            {node.message}
                                        </span>
                                        <span className="history-item-id mono">
                                            {node.id.slice(0, COMMIT_ID_SHORT_LEN)}
                                        </span>
                                        {isHead && (
                                            <span className="history-current-tag">
                                                HEAD
                                            </span>
                                        )}
                                    </div>
                                </div>
                                {isExpanded && (
                                    <div
                                        className="history-expand"
                                        onClick={(e) => e.stopPropagation()}
                                    >
                                        {!hasPrevious && (
                                            <div className="history-expand-empty mono">
                                                No previous commit to diff (root
                                                snapshot).
                                            </div>
                                        )}
                                        {hasPrevious && isLoading && (
                                            <div className="history-expand-empty mono">
                                                Loading changes...
                                            </div>
                                        )}
                                        {hasPrevious &&
                                            !isLoading &&
                                            !expandedDiff && (
                                                <div className="history-expand-empty mono">
                                                    Failed to load changes. Tap
                                                    again to retry.
                                                </div>
                                            )}
                                        {hasPrevious &&
                                            !isLoading &&
                                            expandedDiff &&
                                            expandedDiff.files.length === 0 && (
                                                <div className="history-expand-empty mono">
                                                    No changed files.
                                                </div>
                                            )}
                                        {hasPrevious &&
                                            !isLoading &&
                                            expandedDiff &&
                                            expandedDiff.files.length > 0 && (
                                                <div className="history-expand-list">
                                                    {expandedDiff.files.map(
                                                        (f) => {
                                                            const isActive =
                                                                activeDiffToNodeId ===
                                                                    node.id &&
                                                                selectedDiffPath ===
                                                                    f.path;
                                                            return (
                                                                <button
                                                                    key={`${node.id}-${f.path}`}
                                                                    type="button"
                                                                    className={`history-expand-file${isActive ? " is-active" : ""}`}
                                                                    onClick={() =>
                                                                        onSelectExpandedDiffFile(
                                                                            node.id,
                                                                            f.path,
                                                                        )
                                                                    }
                                                                >
                                                                    <span
                                                                        className={`diff-kind diff-kind-${f.kind}`}
                                                                    >
                                                                        {f.kind}
                                                                    </span>
                                                                    <span className="history-expand-path mono">
                                                                        {f.path}
                                                                    </span>
                                                                    {f.is_binary && (
                                                                        <span className="history-expand-bin">
                                                                            BIN
                                                                        </span>
                                                                    )}
                                                                </button>
                                                            );
                                                        },
                                                    )}
                                                </div>
                                            )}
                                    </div>
                                )}
                                <div className="history-item-meta mono">
                                    {formatNodeTime(node.created_at_unix_ms)}
                                </div>
                            </div>
                        </div>
                    );
                })}
            </div>
            {contextMenu && (
                <div
                    ref={contextMenuRef}
                    className="history-context-menu"
                    style={{ left: contextMenu.x, top: contextMenu.y }}
                    onClick={(e) => e.stopPropagation()}
                >
                    <button
                        type="button"
                        className="history-context-item"
                        disabled={!canContextCheckout}
                        onClick={() => {
                            if (!contextNode || !canContextCheckout) return;
                            setContextMenu(null);
                            void onCheckoutSnapshot(contextNode.id);
                        }}
                    >
                        Checkout Snapshot
                    </button>
                </div>
            )}
        </>
    );
}
