import {DiagramBox, DiagramSectionDrawerBox, PresenceRemainder} from "oxidd-vis-rust";
import {IViewGroup, ViewState} from "../views/ViewState";
import {IPoint} from "../../utils/_types/IPoint";
import {DerivedField} from "../../utils/DerivedField";
import {Field} from "../../watchables/Field";
import {chain} from "../../watchables/mutator/chain";
import {IBaseViewSerialization} from "../_types/IBaseViewSerialization";
import {IDiagramVisualizationSerialization} from "./_types/IDiagramVisualizationSerialization";
import {IMutator} from "../../watchables/mutator/_types/IMutator";
import {Observer} from "../../watchables/Observer";
import {ISharedVisualizationState} from "./_types/ISharedVisualizationState";
import {IRectangle} from "../../utils/_types/IRectangle";
import {ITool} from "../toolbar/_types/ITool";
import {IToolEvent} from "../toolbar/_types/IToolEvent";
import {Derived} from "../../watchables/Derived";
import {binaryToString, stringToBinary} from "../../utils/binarySerialization";
import {ITerminalState} from "./_types/ITerminalState";
import {Mutator} from "../../watchables/mutator/Mutator";
import {getConfigurationObjectWrapper} from "../configuration/getConfigurationObjectWrapper";
import {IWatchable} from "../../watchables/_types/IWatchable";

/** The state of a single visualization of a diagram */
export class DiagramVisualizationState extends ViewState {
    /** The canvas holding the visualization */
    public readonly canvas: HTMLCanvasElement;

    /** The configuration object to change settings of this visualization */
    public readonly config = new Derived(() =>
        getConfigurationObjectWrapper({
            owner: new Derived(watch => [
                {
                    name: watch(this.name),
                    id: this.ID,
                },
            ]),
            config: this.drawer.get_configuration(),
        })
    );

    /** The diagram drawer */
    protected readonly drawer: DiagramSectionDrawerBox;

    /** The start date */
    protected start = Date.now() - 2000; // - 2000 to skip initial transition

    /** The local transformation */
    public readonly transform = new Field({offset: {x: 0, y: 0}, scale: 15});
    protected transformObserver = new Observer(this.transform).add(() =>
        this.sendTransform()
    );

    /** The size of the canvas */
    public readonly size = new Field({x: 0, y: 0});
    protected sizeObserver = new Observer(this.size).add(() => this.sendTransform());

    /** Visualization state shared between visualizations of this diagram */
    public readonly sharedState: ISharedVisualizationState;
    protected selectionObserver: Observer<{
        selected: Uint32Array;
        highlight: Uint32Array;
    }>;

    /** @override */
    public readonly children: IWatchable<ViewState[]> = new Derived(watch =>
        watch(watch(this.config).nonNestedDescendentViews)
    );

    /** @override */
    public readonly groups: IWatchable<IViewGroup[]> = new Derived(watch => [
        {targets: [this.ID, ...watch(this.descendants).map(group => group.ID)]},
    ]);

    /**
     * Creates a new diagram visualization
     * @param drawer The diagram drawer to control
     * @param canvas The canvas that the drawer visualizes in
     * @param sharedState The state shared between visualizations of this diagram
     */
    public constructor(
        drawer: DiagramSectionDrawerBox,
        canvas: HTMLCanvasElement,
        sharedState: ISharedVisualizationState
    ) {
        super();
        this.name.set("Visualization").commit();
        this.category.set("visualization").commit();
        this.drawer = drawer;
        this.canvas = canvas;
        this.sharedState = sharedState;

        this.selectionObserver = new Observer(
            new Derived(watch => ({
                selected: watch(sharedState.selection),
                highlight: watch(sharedState.highlight),
            }))
        ).add(() => this.sendHighlight(), true);
    }

    /** Updates the transform on rust's side */
    protected sendTransform() {
        const transform = this.transform.get();
        const size = this.size.get();
        this.canvas.width = size.x;
        this.canvas.height = size.y;
        this.drawer.set_transform(
            size.x,
            size.y,
            transform.offset.x,
            transform.offset.y,
            transform.scale
        );
    }

    protected sendHighlight() {
        const selectNodes = this.drawer.source_nodes_to_local(
            this.sharedState.selection.get()
        );
        const highlightNodes = this.drawer.source_nodes_to_local(
            this.sharedState.highlight.get()
        );
        this.drawer.set_selected_nodes(selectNodes, highlightNodes);
    }

    /**
     * Updates the diagram's layout
     */
    protected relayout() {
        const layoutStart = Date.now();
        this.drawer.layout(layoutStart - this.start);
        const layoutTime = Date.now() - layoutStart;
        this.start += layoutTime;
    }

    /**
     * Centers the visualization such that it fits exactly in the screen
     * @param maxScale The maximum scale to apply to fill the screen
     * @return The mutator to commit the event
     */
    public fitVisualization(maxScale: number = 20): IMutator<unknown> {
        const bb = this.drawer.get_bounding_box();
        const size = this.size.get();
        const height = bb.y_max - bb.y_min + 1;
        const width = bb.x_max - bb.x_min + 2; // Extra padding on x-axis compared to y
        const scale = Math.min(size.y / height, size.x / width, maxScale);
        const center = {
            x: (bb.x_min + bb.x_max) / 2,
            y: (bb.y_min + bb.y_max) / 2,
        };
        return this.transform.set({
            scale,
            offset: {x: -center.x, y: -center.y},
        });
    }

    /**
     * Applies the given tool to this visualization
     * @param tool The tool to apply
     * @param nodes The nodes to apply it to
     * @param event The event to activate the tool for, defaults to {type: "release"}
     */
    public applyTool(tool: ITool, nodes: Uint32Array, event?: IToolEvent): void {
        const layout = tool.apply(this, this.drawer, nodes, event ?? {type: "release"});
        if (layout) this.relayout();
    }

    /**
     * Retrieves the local visualization node ids in the given rectangle
     * @param area The area for which to retrieve the nodes that lay (partially) inside it, with (0,0) being the top_left of the current view, and (width, height) being the bottom_right of the current view.
     * @returns The ids of the nodes that lay in this area
     */
    public getNodes(area: IRectangle): Uint32Array {
        const canvasArea = this.canvas.getBoundingClientRect();
        const xRel = area.left_top.x / canvasArea.width - 0.5;
        const yRel = area.left_top.y / canvasArea.height - 0.5;
        const widthRel = area.size.x / canvasArea.width;
        const heightRel = area.size.y / canvasArea.height;
        return this.drawer.get_nodes(xRel, -yRel - heightRel, widthRel, heightRel, 2000); // TODO: create selection number setting
    }

    /**
     * Converts the ids of local visualization nodes, to the source node ids (in the overall diagram) that they represent
     * @param nodes The nodes for which to obtain the source ids
     * @return The source ids
     */
    public getNodeSources(nodes: Uint32Array): Uint32Array {
        return this.drawer.local_nodes_to_sources(nodes);
    }

    /** Renders a frame to the canvas */
    public render() {
        const time = Date.now() - this.start;
        this.drawer?.render(time);
    }

    // State management
    /**
     * Disposes the data held by this visualization (drops the rust data)
     */
    public dispose() {
        this.transformObserver.destroy();
        this.sizeObserver.destroy();
        this.selectionObserver.destroy();
        this.config.get().destroy();
        this.drawer.free();
        (this.drawer as any) = undefined;
    }

    /** @override */
    public serialize(): IDiagramVisualizationSerialization {
        const rustState = this.drawer.serialize_state();
        const stateText = binaryToString(rustState);
        return {
            ...super.serialize(),
            transform: this.transform.get(),
            rustState: stateText,
            configuration: this.config.get().serialize(),
        };
    }

    /** @override */
    public deserialize(data: IDiagramVisualizationSerialization): IMutator<unknown> {
        return chain(push => {
            push(super.deserialize(data));
            push(this.transform.set(data.transform));
            const rustState = stringToBinary(data.rustState);
            this.drawer.deserialize_state(rustState);
            push(this.config.get().deserialize(data.configuration as never));
        });
    }
}
