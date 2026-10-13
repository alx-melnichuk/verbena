import { CommonModule, NgTemplateOutlet } from "@angular/common";
import {
    ChangeDetectionStrategy, Component, ContentChild, EventEmitter, Input, Output, TemplateRef, ViewChild, ViewEncapsulation
} from "@angular/core";
import { MAT_MENU_DEFAULT_OPTIONS, MatMenuItem, MatMenuModule, MatMenuPanel } from "@angular/material/menu";
import { RouterLink } from "@angular/router";

export interface MenuItemInfo {
    code?: string | undefined;
    disabled?: boolean | undefined;
    label: string;
    panelClass?: string | undefined;
    route?: string | undefined;
    subItems?: MenuItemInfo[];
}

export class MenuItemInfoUtil {
    public static createItem(item: Partial<MenuItemInfo>): MenuItemInfo {
        return {
            code: item.code,
            disabled: item.disabled,
            label: item.label || "",
            route: item.route,
            subItems: MenuItemInfoUtil.createItems(item.subItems || []),
        };
    }
    public static createItems(items: Partial<MenuItemInfo>[]): MenuItemInfo[] {
        const result: MenuItemInfo[] = [];
        for (let idx = 0; idx < items.length; idx++) {
            result.push(MenuItemInfoUtil.createItem(items[idx]));
        }
        return result;
    }
}

@Component({
    selector: "app-menu-item",
    exportAs: "appMenuItem",
    standalone: true,
    imports: [CommonModule, MatMenuModule, NgTemplateOutlet, RouterLink],
    templateUrl: "./menu-item.html",
    styleUrl: "./menu-item.scss",
    encapsulation: ViewEncapsulation.None,
    changeDetection: ChangeDetectionStrategy.OnPush,
    providers: [
        {
            provide: MAT_MENU_DEFAULT_OPTIONS,
            useValue: {
                overlapTrigger: false,
                xPosition: 'after',
                yPosition: 'below',
                backdropClass: 'cdk-overlay-transparent-backdrop',
                overlayPanelClass: 'mi-overlay-panel'
            }
        }
    ]
})
export class MenuItem {
    @Input()
    public items: MenuItemInfo[] | undefined = [];
    @Input()
    public tmplMenuItem: TemplateRef<unknown> | undefined;

    @Output()
    readonly selected: EventEmitter<MenuItemInfo> = new EventEmitter();

    @ViewChild("childMenu")
    public childMenu: MatMenuPanel<MatMenuItem> | undefined;

    @ContentChild('tmplMenuItem', { static: true })
    public tmplMenuItemRef: TemplateRef<unknown> | undefined;

    // ** Public API **

    public doSelected(item: MenuItemInfo): void {
        this.selected.emit(item);
    }

    public clickItem(item: MenuItemInfo): void {
        if (!item || !!item.disabled) {
            return;
        }
        const navItem: MenuItemInfo = { code: item.code, label: item.label, route: item.route };
        this.selected.emit(navItem);
    }
}
