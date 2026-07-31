import React, {FC, useCallback, useEffect, useLayoutEffect, useRef} from "react";
import {DiagramVisualizationState} from "../../../state/diagrams/DiagramVisualizationState";
import {useTransformCallbacks} from "./useTransformCallbacks";
import {css} from "@emotion/css";
import {ViewContainer} from "../../components/layout/ViewContainer";
import {BoxSelection} from "./BoxSelection";
import {useToolbar} from "../../providers/ToolbarContext";
import {useWatch} from "../../../watchables/react/useWatch";
import {ActionButton, PrimaryButton, useTheme} from "@fluentui/react";
import {Toolbar} from "../toolbar/Toolbar";
import {PresenceRemainder} from "oxidd-vis-rust";
import {ConfigTypeComp} from "../../components/configuration/ConfigTypeComp";

export const DiagramVisualization: FC<{visualization: DiagramVisualizationState}> = ({
    visualization,
}) => {
    const theme = useTheme();
    const watch = useWatch();
    const toolbar = useToolbar();

    const dispose_ref = useRef(() => {});
    useEffect(() => () => dispose_ref.current(), []);

    // Use a ref callback for immediate rendering without flicker
    const el_ref = (el: HTMLDivElement) => {
        if (!el) return;
        const setSize = (center = false) => {
            const width = el.clientWidth;
            const height = el.clientHeight;
            if (width <= 0 || height <= 0) return;

            const size = {x: width, y: height};
            const pos = visualization.transform.get().offset;
            // Only center on the first render, we use the visualization being centered as an over-approximation of "first-render"
            if (center && pos.x == 0 && pos.y == 0) {
                visualization.size
                    .set(size)
                    .chain(() => visualization.fitVisualization())
                    .commit();
            } else {
                visualization.size.set(size).commit();
            }
        };
        let running = true;
        const render = () => {
            if (!running) return;
            visualization.render();
            requestAnimationFrame(render);
        };

        el.insertBefore(visualization.canvas, el.firstChild);

        const resizeObserver = new ResizeObserver(() => setTimeout(setSize)); // timeout used to prevent UI updates resulting from UI size change
        resizeObserver.observe(el);

        setSize(true);
        render();

        dispose_ref.current();
        dispose_ref.current = () => {
            running = false;
            resizeObserver.disconnect();
        };
    };

    // Prevent dragging the window when clicking a button
    const preventDrag = useCallback((e: React.MouseEvent) => {
        e.stopPropagation();
    }, []);
    const moveListeners = useTransformCallbacks(visualization.transform);
    return (
        <ViewContainer
            onContextMenu={e => e.preventDefault()}
            ref={el_ref}
            {...moveListeners}
            css={{padding: 0, overflow: "hidden"}}>
            <BoxSelection
                onStart={m => m.buttons == 1}
                onHighlight={(rect, e) => {
                    const nodes = visualization.getNodes(rect);
                    visualization.applyTool(toolbar, nodes, {
                        type: "drag",
                        event: e,
                    });
                }}
                onSelect={(rect, e) => {
                    const nodes = visualization.getNodes(rect);
                    visualization.applyTool(toolbar, nodes, {
                        type: "release",
                        event: e,
                    });
                }}>
                <div
                    onMouseDown={preventDrag}
                    className={css({
                        position: "absolute",
                        right: theme.spacing.m,
                        top: theme.spacing.m,
                        background: theme.palette.neutralLight,
                    })}>
                    <Toolbar toolbar={toolbar} visualization={visualization} />
                </div>
                <ConfigTypeComp value={watch(visualization.config)} />
            </BoxSelection>
        </ViewContainer>
    );
};
