import {useCallback, useEffect, useRef, useState} from "react";
import {ITransition, transition} from "../../../utils/transition";
import {Field} from "../../../watchables/Field";
import {ITransformation} from "../../../state/diagrams/_types/IDiagramVisualizationSerialization";
import {Observer} from "../../../watchables/Observer";

export function useTransformCallbacks(
    transform: Field<ITransformation>
): IInteractionHandlers {
    const scaleTarget = useRef(transform.get().scale);
    const prevTransition = useRef<ITransition | null>(null);

    useEffect(() => {
        const observer = new Observer(transform).add(transform => {
            if (prevTransition.current?.finished ?? true) {
                scaleTarget.current = transform.scale;
            }
        });
        return () => observer.destroy();
    }, []);
    const getTargetPoint = (e: React.WheelEvent<HTMLElement>) => {
        const canvas = e.target as HTMLElement;
        const bound = canvas.getBoundingClientRect();
        const elX = e.pageX - bound.x;
        const elY = e.pageY - bound.y;
        return {
            x: elX - 0.5 * bound.width,
            y: -(elY - 0.5 * bound.height),
        };
    };
    const setScale = (newScale: number, target: IPoint) => {
        const {
            offset: {x, y},
            scale,
        } = transform.get();
        return transform.set({
            offset: {
                x: x + (target.x / newScale - target.x / scale),
                y: y + (target.y / newScale - target.y / scale),
            },
            scale: newScale,
        });
    };

    return {
        onWheel: useCallback(e => {
            const delta = e.deltaX + e.deltaY + e.deltaZ;
            const multiplier = 1.3 ** (-delta / 100);
            const target = getTargetPoint(e);
            const startScale = transform.get().scale;
            const endScale = (scaleTarget.current *= multiplier);

            e.stopPropagation();
            prevTransition.current?.cancel();
            prevTransition.current = transition(per => {
                setScale((1 - per) * startScale + per * endScale, target).commit();
            }, 100);
        }, []),
        onMouseDown: useCallback(e => {
            if (e.button == 0) return;
            // We register listeners on the window, such that dragging works even when leaving the canvas
            const moveListener = (e: MouseEvent) => {
                if ((e.buttons & 3) != 0) {
                    const {
                        offset: {x, y},
                        scale,
                    } = transform.get();
                    transform
                        .set({
                            offset: {
                                x: x + e.movementX / scale,
                                y: y - e.movementY / scale,
                            },
                            scale,
                        })
                        .commit();
                }
            };
            const upListener = (e: MouseEvent) => {
                window.removeEventListener("mousemove", moveListener);
                window.removeEventListener("mouseup", upListener);
            };
            window.addEventListener("mousemove", moveListener);
            window.addEventListener("mouseup", upListener);
        }, []),
    };
}

type IPoint = {x: number; y: number};
type IInteractionHandlers = {
    onWheel: React.WheelEventHandler<HTMLElement>;
    onMouseDown: (e: React.MouseEvent<HTMLElement, MouseEvent>) => void;
    // onMouseUp: (e: React.MouseEvent<HTMLCanvasElement, MouseEvent>) => void;
    // onMouseMove: (e: React.MouseEvent<HTMLCanvasElement, MouseEvent>) => void;
};
