import { CommonModule, NgTemplateOutlet } from "@angular/common";
import {
    ChangeDetectionStrategy, Component, ContentChild, EventEmitter, Input, Output, TemplateRef, ViewChild, ViewEncapsulation
} from "@angular/core";
import { MatButtonModule } from "@angular/material/button";
import { MatMenu, MatMenuItem, MatMenuModule, MatMenuPanel, MatMenuTrigger } from "@angular/material/menu";
import { RouterLink } from "@angular/router";
import { MenuItem, MenuItemInfo } from "../menu-item/menu-item";

@Component({
    selector: "app-menu-bar",
    exportAs: "appMenuBar",
    standalone: true,
    imports: [CommonModule, MatButtonModule, MatMenuModule, NgTemplateOutlet, RouterLink, MenuItem,],
    templateUrl: "./menu-bar.html",
    styleUrl: "./menu-bar.scss",
    encapsulation: ViewEncapsulation.None,
    changeDetection: ChangeDetectionStrategy.OnPush,
})
export class MenuBar {
    @Input()
    public item: MenuItemInfo | undefined | null;
    @Input()
    public tmplMenuBar: TemplateRef<unknown> | undefined;
    @Input()
    public tmplMenuItem: TemplateRef<unknown> | undefined;

    @Output()
    readonly selected: EventEmitter<MenuItemInfo> = new EventEmitter();

    @ContentChild('tmplMenuBar', { static: true })
    public tmplMenuBarRef: TemplateRef<unknown> | undefined;
    @ContentChild('tmplMenuItem', { static: true })
    public tmplMenuItemRef: TemplateRef<unknown> | undefined;

    @ViewChild(MatMenuTrigger)
    public trigger: MatMenuTrigger | undefined;

    // ** Public API **

    public getChildMenu(value: MatMenuPanel<MatMenuItem> | MatMenu | null | undefined): MatMenuPanel<MatMenuItem> | null {
        return value instanceof MatMenu ? value : null;
    }

    public doSelected(item: MenuItemInfo): void {
        this.selected.emit(item);
    }

    public clickItem(item: MenuItemInfo): void {
        if (!item || !!item.disabled) {
            return;
        }
        const itemInfo: MenuItemInfo = { code: item.code, label: item.label, route: item.route };
        this.selected.emit(itemInfo);
    }
}

