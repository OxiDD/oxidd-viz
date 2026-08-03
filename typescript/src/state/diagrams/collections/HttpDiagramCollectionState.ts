import {MessageBarType} from "@fluentui/react";
import {IWatchable} from "../../../watchables/_types/IWatchable";
import {IMutator} from "../../../watchables/mutator/_types/IMutator";
import {ICollectionStatus, IDiagramCollection} from "../_types/IDiagramCollection";
import {DiagramState} from "../DiagramState";
import {v4 as uuid} from "uuid";
import {Derived} from "../../../watchables/Derived";
import {IHttpDiagramCollectionSerialization} from "./_types/IHttpDiagramCollectionSerialization";
import {chain} from "../../../watchables/mutator/chain";
import {dummyMutator} from "../../../watchables/mutator/Mutator";
import {Field} from "../../../watchables/Field";
import {Constant} from "../../../watchables/Constant";
import {createDiagramBox} from "../createDiagramBox";
import {IDiagramType} from "../_types/IDiagramTypeSerialization";
import {ViewState} from "../../views/ViewState";
import {Observer} from "../../../watchables/Observer";
import {all} from "../../../watchables/mutator/all";
import {ManualDiagramCollectionState} from "./ManualDiagramCollectionState";
import {DiagramCollectionBaseState} from "./DiagramCollectionBaseState";

export class HttpDiagramCollectionState extends DiagramCollectionBaseState {
    protected readonly _status = new Field<ICollectionStatus>(undefined);

    /** The current status of the collection */
    public readonly status: IWatchable<{text: string; type: MessageBarType} | undefined> =
        this._status.readonly();

    /** All the diagrams that can be reached from this collection */
    public readonly descendentViews = new Derived(watch => [
        ...watch(this.diagrams),
        ...watch(this.collections).flatMap(col => watch(col.descendentViews)),
        this.autoOpenTarget,
    ]);

    /** The url of the host we are trying to access */
    public readonly host: string;

    /** Whether to replace old diagrams when opening a new diagram with the same name */
    public readonly replaceOld = new Field<boolean>(true);

    /** The id of the poll interval */
    protected pollID: number;

    /** The number of polls we have done */
    protected pollNum: number = 0;

    /** A target view to open new diagrams in */
    public readonly autoOpenTarget: HttpDiagramCollectionTargetState;

    /**
     * Creates a new http collection that attempts to synchronize with the given source
     * @param host
     */
    public constructor(host: string) {
        super();
        this.host = host;
        this.autoOpenTarget = new HttpDiagramCollectionTargetState(host, this._diagrams);
        this.startPoll();
    }

    /** Starts polling on a regular interval */
    protected startPoll() {
        let awaitingResponse = false;
        this.pollID = setInterval(() => {
            if (!awaitingResponse) {
                awaitingResponse = true;
                this.poll().finally(() => {
                    awaitingResponse = false;
                });
            }
        }, 200) as any;
    }

    /** @override */
    public dispose(): void {
        super.dispose();
        clearInterval(this.pollID);
    }

    /** Performs a single poll */
    protected async poll() {
        this.pollNum++;
        // Check if the host can be reached first
        try {
            await fetch(`${this.host}`, {method: "HEAD", mode: "no-cors"});
        } catch (error) {
            return;
        }

        // Obtain the data from the host
        let diagrams;
        try {
            const diagramsText = await fetch(`${this.host}/diagrams`);
            diagrams = (await diagramsText.json()) as {
                name: string;
                type: IDiagramType;
                diagram: string;
            }[];
        } catch (e) {
            this._status
                .set({
                    type: MessageBarType.error,
                    text: "Unable to obtain diagrams from host",
                })
                .commit();
            return;
        }

        chain(push => {
            try {
                for (const {name, type, diagram} of diagrams) {
                    const oldDiagramState = this.diagrams
                        .get()
                        .find(d => d.sourceName.get() == name);
                    if (oldDiagramState && this.replaceOld.get()) {
                        push(this.removeDiagram(oldDiagramState));
                    }

                    const diagramBox = createDiagramBox(type);
                    const diagramState = new DiagramState(diagramBox, type);
                    push(diagramState.sourceName.set(name));
                    push(diagramState.name.set(name + " diagram"));
                    push(diagramState.createSectionFromDDDMP(diagram, name));
                    push(this._diagrams.set([...this.diagrams.get(), diagramState]));
                }

                if (this._status.get() != undefined) push(this._status.set(undefined));
            } catch (e) {
                console.error(e);
                push(
                    this._status.set({
                        type: MessageBarType.error,
                        text: "An error occurred while loading diagrams",
                    })
                );
            }
        }).commit();
    }

    /** @override */
    public serialize(): IHttpDiagramCollectionSerialization {
        return {
            ...super.serialize(),
            host: this.host,
            replaceOld: this.replaceOld.get(),
            target: this.autoOpenTarget.serialize(),
        };
    }

    /** @override */
    public deserialize(data: IHttpDiagramCollectionSerialization): IMutator {
        return chain(push => {
            push(super.deserialize(data));
            (this.host as any) = data.host;
            push(this.replaceOld.set(data.replaceOld));
            push(this.autoOpenTarget.deserialize(data.target));
        });
    }
}

/**
 * A view state class used to open diagrams in for a given http diagram collection
 */
export class HttpDiagramCollectionTargetState extends ViewState {
    /** The url of the host we are trying to access */
    public readonly host: string;

    /** The diagrams that we are observing */
    protected diagrams: IWatchable<DiagramState[]>;

    protected listeners: ((diagram: DiagramState) => void | IMutator)[] = [];

    protected observer: Observer<DiagramState[]>;

    /** Creates the diagram collection target for a given set of diagrams */
    public constructor(host: string, diagrams: IWatchable<DiagramState[]>) {
        super();
        this.name.set(`${host} target`).commit();
        this.host = host;
        this.diagrams = diagrams;

        this.observer = new Observer(this.diagrams).add((diagrams, prev) => {
            let mutators = [];
            for (const diagram of diagrams) {
                const alreadyExisted = prev.some(d => d == diagram);
                if (alreadyExisted) continue;

                for (const listener of this.listeners) {
                    console.log(diagram);
                    const mutator = listener(diagram);
                    if (mutator) mutators.push(mutator);
                }
            }

            all(mutators).commit();
        });
    }

    /**
     * Registers a new listener for diagrams being opened
     * @param listener The listener to register
     * @returns The function that can be called for removing the listener
     */
    public onDiagramOpen(
        listener: (diagram: DiagramState) => void | IMutator
    ): () => void {
        this.listeners.push(listener);
        return () => {
            const index = this.listeners.indexOf(listener);
            if (index != -1) {
                this.listeners.splice(index, 1);
            }
        };
    }
}
