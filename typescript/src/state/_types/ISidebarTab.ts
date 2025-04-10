import {ViewState} from "../views/ViewState";

/** The sidebar tab data */
export type ISidebarTab = {
    icon: string;
    name: string;
    view: ViewState;
    hidden?: boolean;
    skipSerialization?: boolean;
};
