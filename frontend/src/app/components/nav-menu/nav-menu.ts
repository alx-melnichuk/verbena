import { CommonModule } from "@angular/common";
import {
    ChangeDetectionStrategy, Component, ContentChild, EventEmitter, Input, Output, TemplateRef, ViewEncapsulation
} from "@angular/core";
import { MenuBar } from "../menu-bar/menu-bar";
import { MenuItemInfo } from "../menu-item/menu-item";

@Component({
    selector: "app-nav-menu",
    exportAs: "appNavMenu",
    standalone: true,
    imports: [CommonModule, MenuBar],
    templateUrl: "./nav-menu.html",
    styleUrl: "./nav-menu.scss",
    encapsulation: ViewEncapsulation.None,
    changeDetection: ChangeDetectionStrategy.OnPush,
})
export class NavMenu {
    @Input()
    public items: MenuItemInfo[] = [];
    @Input()
    public tmplMenuBar: TemplateRef<unknown> | undefined;
    @Input()
    public tmplMenuItem: TemplateRef<unknown> | undefined;

    @Output()
    readonly selected: EventEmitter<MenuItemInfo> = new EventEmitter();

    @ContentChild("tmplMenuBar", { static: true })
    public tmplMenuBarRef: TemplateRef<unknown> | undefined;

    @ContentChild("tmplMenuItem", { static: true })
    public tmplMenuItemRef: TemplateRef<unknown> | undefined;

    // ** Public API **

    public doSelected(item: MenuItemInfo): void {
        this.selected.emit(item);
    }

    // ** Private API **

}
