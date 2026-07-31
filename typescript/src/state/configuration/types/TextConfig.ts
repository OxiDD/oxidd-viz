import {AbstractConfigurationObject} from "oxidd-vis-rust";
import {IWatchable} from "../../../watchables/_types/IWatchable";
import {ConfigurationObject, IOwnedAbstractConfig} from "../ConfigurationObject";
import {Derived} from "../../../watchables/Derived";
import {IMutator} from "../../../watchables/mutator/_types/IMutator";
import {chain} from "../../../watchables/mutator/chain";
import {IRunnable} from "../../../watchables/_types/IRunnable";

/**
 * A configuration object for free choices
 */
export class TextConfig
    extends ConfigurationObject<{options: string[]; value: string}>
    implements IWatchable<string>
{
    /** The options of the choice  */
    public readonly options = new Derived<string[]>(watch => watch(this._value).options);

    /** The currently selected option (text) */
    public readonly selected = new Derived<string>(watch => watch(this._value).value);

    /**
     * Creates a new config object
     * @param object The rust configuration that represents a choice
     */
    public constructor(object: IOwnedAbstractConfig) {
        super(object);
    }

    /**
     * Sets the new value
     * @param value The new value
     * @returns The mutator to commit the change
     */
    public set(value: string): IMutator {
        return this.setValue({
            options: this.options.get(),
            value,
        });
    }

    /**
     * Sets the options of the input
     * @param values THe new options
     * @returns The mutator to commit the change
     */
    public setOptions(values: string[]): IMutator {
        return this.setValue({
            options: ["none", ...values],
            value: this.selected.get(),
        });
    }

    /** @override */
    public get(): string {
        return this.selected.get();
    }
    /** @override */
    public onDirty(listener: IRunnable): IRunnable {
        return this.selected.onDirty(listener);
    }
    /** @override */
    public onChange(listener: IRunnable): IRunnable {
        return this.selected.onChange(listener);
    }
}
