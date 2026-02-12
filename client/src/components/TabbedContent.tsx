import React, { ReactElement, ReactNode, useState } from "react";
import { Button } from "./Button";
import "./tabbedContent.css";

export interface TabDefinition<T extends string> {
    id: T;
    label: ReactNode;
    content: ReactElement;
}

interface TabbedContentProps<T extends string> {
    tabs: TabDefinition<T>[];
    defaultTab?: T;
    className?: string;
}

export function TabbedContent<T extends string>(props: TabbedContentProps<T>): ReactElement {
    const [activeTab, setActiveTab] = useState<T>(props.defaultTab ?? props.tabs[0].id);

    return (
        <div className={`tabbed-content ${props.className ?? ""}`}>
            <div className="settings-tabs">
                {props.tabs.map(tab => (
                    <Button
                        key={tab.id}
                        highlighted={activeTab === tab.id}
                        onClick={() => setActiveTab(tab.id)}
                    >
                        {tab.label}
                    </Button>
                ))}
            </div>
            <div className="tab-content">
                {props.tabs.find(tab => tab.id === activeTab)?.content}
            </div>
        </div>
    );
}
