import {Icon, Link, useTheme} from "@fluentui/react";
import React, {FC} from "react";
import {ViewState} from "../../../state/views/ViewState";
import {AppState} from "../../../state/AppState";
import {css} from "@emotion/css";
import {ViewContainer} from "../../components/layout/ViewContainer";
import {CenteredContainer} from "../../components/layout/CenteredContainer";
import SyntaxHighlighter from "react-syntax-highlighter";
import {a11yDark, vs2015} from "react-syntax-highlighter/dist/esm/styles/hljs";

export const Info: FC<{app: AppState}> = ({app}) => {
    const theme = useTheme();
    const link = (text: string, openView: ViewState) => (
        <Link onClick={() => app.views.open(openView).commit()}>{text}</Link>
    );

    return (
        <CenteredContainer>
            <h1 style={{color: theme.palette.themePrimary, fontSize: 40, marginTop: 10}}>
                <Icon
                    iconName="GitGraph"
                    styles={{
                        root: {marginRight: 10, fontSize: 45, verticalAlign: "bottom"},
                    }}
                />
                OxiDD-vis
            </h1>
            <p>
                OxiDD-vis is a Decision Diagram (DD) visualization tool. It is in active
                development, and more features will be added over time. Two types of
                diagrams are currently supported:
                <ul>
                    <li>Binary Decision Diagrams without complement edges</li>
                    <li>
                        Multi-Terminal Binary Decision Diagrams with numeric terminals
                    </li>
                </ul>
            </p>

            <h2>Loading diagrams</h2>
            <p>
                There are two ways to add diagrams:{" "}
                <ul>
                    <li>Local: Manually adding a diagram + loading nodes from a file</li>
                    <li>Remote: Adding a remote diagram source</li>
                </ul>
            </p>
            <h3>Local</h3>
            <p>
                To add a local diagram, simply open the{" "}
                {link("diagrams panel", app.diagrams)} and click "Add local BDD" or "Add
                local MTBDD". This should add a shared diagram, which you can now add
                content into using either "Load from DDDMP" or 'Load from BuDDy'. Here
                you can either select a file or supply text contents, and load the
                diagrams. After the diagrams finished loading, a new section should appear
                in the diagram. Clicking this section opens the visualization. Any diagram
                allows new sections to be created by selecting a node in the visualization
                and clicking the "Create from selection" button.
            </p>
            <h3>Remote</h3>
            <p>
                To add a remote diagram source, simply open the{" "}
                {link("diagrams panel", app.diagrams)} and click "Add diagram source".
                This opens up a window in which you can select the host of the diagrams
                you want to show. In most cases, this host will be the default entered
                address (configured as the default in OxiDD). After the host is added, it
                will be regularly check for new diagrams. <br />
                Here you can also select "Automatically open diagrams" to open a tab in
                which the diagrams will be opened when created, and toggle whether new
                diagrams should replace previous diagrams by the same name.
                <br />
            </p>
            <p>
                Remote diagrams can be provided by any tool which hosts a server with a{" "}
                <code>/diagrams</code> path, which provides a JSON response of the
                following format:
                <SyntaxHighlighter language="javascript" style={vs2015}>
                    {`{\n\tname: string;\n\ttype: "BDD"|"MTBDD";\n\tdiagram: string;\n}[]`}
                </SyntaxHighlighter>
                The diagram should be the contents of a valid DDDMP file with the given
                type. When no new diagrams have been created, this request should return a
                404. OxiDD provides a visualize function in the <code>oxidd-dump</code>{" "}
                crate which temporarily hosts such a server until the contents are read by
                OxiDD-vis.
            </p>

            <h2>Features</h2>
            <p>
                Despite being unfinished, OxiDD-vis already includes several great
                features. Some of these will be briefly introduced here.
            </p>
            <h3>Tabs</h3>
            <p>
                The layout of the tool can be customized by dragging tabs around. Simply
                drag a tab and drop it in one of the highlighted blue areas to split the
                corresponding section into two. This allows for viewing several diagrams
                side by side. These tabs can also be renamed by right-clicking them.
            </p>
            <h3>Terminal hiding</h3>
            <p>
                By default the false terminals are hidden in the diagram. This means that
                if a node only has 1 outgoing edge, the other edge goes to the false node.
                For either terminal, a dropdown in the bottom right of the diagram
                controls whether it is visible. Instead of showing it as a single node,
                you can also choose to duplicate it once per parent node.
            </p>
            <h3>Node grouping</h3>
            <p>
                To deal with large diagrams, nodes can be grouped together. By default, if
                a loaded diagram is too large to effectively display, its nodes remain
                grouped in a single group. The tools in the top right can be used to
                either group a selection of nodes (by dragging on screen), or ungroup the
                children of a selection of nodes.
            </p>
            <h3>Settings</h3>
            <p>
                Different decision diagrams provide different settings. Every
                visualization contains a gear icon in the bottom right, which opens a
                dedicated settings panel for the corresponding visualization.
            </p>

            {/* <p>Settings: {link("settings", app.settings)}</p> */}
        </CenteredContainer>
    );
};
