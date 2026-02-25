import { IconButton } from "../ui";

export type ActivityView = "explorer" | "search" | "history" | "lab";

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

type ActivityBarProps = {
    activeView: ActivityView;
    onSelectActivity: (view: ActivityView) => void;
};

export function ActivityBar({ activeView, onSelectActivity }: ActivityBarProps) {
    return (
        <aside className="activity-bar">
            <div className="activity-list">
                {ACTIVITY_ITEMS.map((item) => (
                    <IconButton
                        key={item.id}
                        icon={item.icon}
                        iconClassName="activity-icon"
                        className={`activity-btn ${activeView === item.id ? "is-active" : ""}`}
                        onClick={() => onSelectActivity(item.id)}
                        title={item.title}
                        unstyled
                        dot={item.hasDot}
                    />
                ))}

                <div className="activity-spacer" />

                <IconButton
                    icon="account_circle"
                    iconClassName="activity-icon"
                    className="activity-btn"
                    title="Account"
                    unstyled
                />
                <IconButton
                    icon="settings"
                    iconClassName="activity-icon"
                    className="activity-btn"
                    title="Settings"
                    unstyled
                />
            </div>
        </aside>
    );
}
