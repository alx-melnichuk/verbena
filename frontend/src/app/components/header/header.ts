import { CommonModule } from "@angular/common";
import {
    ChangeDetectionStrategy, ChangeDetectorRef, Component, ElementRef, EventEmitter, HostBinding, HostListener, inject, Input, OnChanges,
    Output, SimpleChanges, ViewEncapsulation
} from "@angular/core";
import { FormsModule } from "@angular/forms";
import { MatButtonModule } from "@angular/material/button";
import { MatMenuModule } from "@angular/material/menu";
import { MatSlideToggleModule } from "@angular/material/slide-toggle";
import { MatToolbarModule } from "@angular/material/toolbar";
import { TranslatePipe } from "@ngx-translate/core";
import { LOCALE_LIST } from "../../common/locale-srv";
import { MainMenuUtil, ItemData } from "../../common/main-menu";
import { MAIN_MENU_LIST } from "../../common/routes";
import { User } from "../../common/session-srv";
import { THEME_LIST } from "../../common/theme-svr";
import { ReplaceWithZeroUtil } from "../../utils/replace-with-zero.util";
import { StringUtil } from "../../utils/string.util";
import { Image } from "../image/image";
import { MenuBar } from "../menu-bar/menu-bar";
import { MenuItem, MenuItemInfo } from "../menu-item/menu-item";

export const HM_LOGOUT = "logout";
export const HM_SET_LOCALE = "setLocale";
export const HM_SET_COLOR_SCHEME = "setColorScheme";

const MIN_WINDOW_WIDTH = 850; // Minimum window width for displaying the main menu.
const RESIZE_TIMEOUT = 50; // milliseconds
const LOGOUT = "logout";

@Component({
    selector: "app-header",
    exportAs: "appHeader",
    standalone: true,
    imports: [CommonModule, FormsModule, MatButtonModule, MatMenuModule, MatSlideToggleModule, MatToolbarModule,
        TranslatePipe, Image, MenuBar, MenuItem],
    templateUrl: "./header.html",
    styleUrl: "./header.scss",
    encapsulation: ViewEncapsulation.None,
    changeDetection: ChangeDetectionStrategy.OnPush,
})
export class Header implements OnChanges {
    private changeDetector: ChangeDetectorRef = inject(ChangeDetectorRef);
    private readonly elementRef = inject<ElementRef<HTMLElement>>(ElementRef);

    @Input()
    public currentRoute: string | null = null;
    @Input()
    public locale: string | null = null;
    @Input()
    public user: User | null = null;
    @Input()
    public currentTheme: string | null = null;

    @Output()
    readonly command: EventEmitter<Record<string, string>> = new EventEmitter();

    private isHideMainMenu: boolean = false;
    private menuMain: MenuItemInfo[] = [];
    public menuMainItems: MenuItemInfo[] = [];
    private menuShort: MenuItemInfo[] = [];
    public menuShortItems: MenuItemInfo[] = [];
    private timerResizeEvent: number | null = null;
    public userNameSymbols: string = "";
    public userNickname: string = "";

    @HostBinding("class.h-is-authorized")
    get isAuthorizedVal(): boolean { return !!this.user; }

    @HostListener("window:resize", ["$event"])
    public onResize(event: Event): void {
        if (this.timerResizeEvent == null) {
            const isHideMainMenu = !this.getIsShowMainMenu();
            if (this.isHideMainMenu != isHideMainMenu) {
                this.isHideMainMenu = isHideMainMenu;
                this.menuMainItems = !this.isHideMainMenu ? this.menuMain.slice() : [];
                const listInit = !this.isHideMainMenu ? [] : this.menuMain.slice();
                this.menuShortItems = listInit.concat(this.menuShort.slice());
                this.changeDetector.markForCheck();
            }
        }

        if (this.timerResizeEvent != null) {
            clearTimeout(this.timerResizeEvent);
        }
        this.timerResizeEvent = setTimeout(() => {
            this.timerResizeEvent = null;
        }, RESIZE_TIMEOUT);
    }

    ngOnChanges(changes: SimpleChanges): void {
        this.isHideMainMenu = !this.getIsShowMainMenu();
        if (!!changes["user"]) {
            this.userNickname = ReplaceWithZeroUtil.replace(this.user?.nickname);
            this.userNameSymbols = (StringUtil.capitalizeOnlyFirstLetter(this.user?.nickname) || "").slice(0, 2);
            this.menuMain = this.getMainMenu(MAIN_MENU_LIST, !!this.user, MainMenuUtil.getPrefix());
            this.menuMainItems = !this.isHideMainMenu ? this.menuMain.slice() : [];
        }
        if (!!changes["user"] || !!changes["currentRoute"]) {
            this.menuMain = this.updateByCurrRoute(this.menuMain, this.currentRoute);
            this.menuMainItems = !this.isHideMainMenu ? this.menuMain.slice() : [];
            this.changeDetector.markForCheck();
        }
        if (!!changes["currentTheme"] || !!changes["locale"]) {
            const listInit = !this.isHideMainMenu ? [] : this.menuMain.slice();
            this.menuShort = this.getShortMenu(this.currentTheme || "", this.locale || "");
            this.menuShortItems = listInit.concat(this.menuShort.slice());
        }
    }

    // ** Public API **

    public doSelected(item: MenuItemInfo): void {
        const code = item.code || "";
        if (LOCALE_LIST.includes(code)) {
            this.doCommand(HM_SET_LOCALE, code);
        } else if (code.startsWith("light-") || code.startsWith("dark-")) {
            this.doCommand(HM_SET_COLOR_SCHEME, code);
        } else if (code == LOGOUT) {
            this.doCommand(HM_LOGOUT, "");
        }
    }

    public doLogout(): void {
        this.doCommand(HM_LOGOUT, "");
    }

    public doSetLocale(value: string): void {
        this.doCommand(HM_SET_LOCALE, value);
    }

    public doSetColorScheme(value: string): void {
        this.doCommand(HM_SET_COLOR_SCHEME, value);
    }

    // ** Private API **

    private doCommand(commandName: string, commandValue: string): void {
        this.command.emit({ [commandName]: commandValue });
    }

    private getWidth(): number {
        return this.elementRef.nativeElement.offsetWidth;
    }
    private getIsShowMainMenu(): boolean {
        return MIN_WINDOW_WIDTH < this.getWidth();
    }

    private mapItemDataToMenuItemInfo(itemsData: ItemData[]): MenuItemInfo[] {
        let nodes: { [key: number]: MenuItemInfo } = {};

        const nodeMap = itemsData.reduce((res, item) => {
            res[item.id] = { label: item.label, route: item.route, subItems: [] };
            return res;
        }, nodes);

        const result: MenuItemInfo[] = [];
        itemsData.forEach((v) => (v.ownId === null ? result.push(nodeMap[v.id]) : nodeMap[v.ownId].subItems?.push(nodeMap[v.id])));
        return result;
    }
    private getMainMenu(list: string[], isAuth: boolean, prefix: string): MenuItemInfo[] {
        const list1 = MainMenuUtil.getListByAuth(list, isAuth).map((v) => v.slice(prefix.length));
        const list2 = MainMenuUtil.getItemDataList(list1, "/");
        const list3 = list2.map((v) => ({
            ...v
            , ...{ label: (!!v.route ? "main_menu." + v.route.replaceAll("/", "_") : v.label) }
            , ...{ route: (!!v.route ? prefix + v.route : "") }
        }));
        return this.mapItemDataToMenuItemInfo(list3);
    }

    private updateByCurrRoute(items: MenuItemInfo[], currentRoute: string | null): MenuItemInfo[] {
        const result: MenuItemInfo[] = [];
        const currRoute = currentRoute || "";
        for (let idx = 0; idx < items.length; idx++) {
            const item = items[idx];
            const panelClass = "h-elem-menu" + (currRoute.startsWith(item.route || "") ? " h-selected" : "");
            result.push({ ...item, ...{ panelClass } });
        }
        return result;
    }

    private getMenuItems(list: string[], prefix: string, codeValue: string): MenuItemInfo {
        const items = list.map((v) => ({
            code: v, label: (prefix + "." + v), panelClass: (v == codeValue ? "check" : undefined), subItems: []
        }));
        return { label: "main_menu." + prefix, subItems: items };
    }
    private getShortMenu(currTheme: string, currLocale: string): MenuItemInfo[] {
        const result: MenuItemInfo[] = [];

        const themeItem = this.getMenuItems([...THEME_LIST], "theme", currTheme);
        result.push(themeItem);

        const localeItem = this.getMenuItems([...LOCALE_LIST], "locale", currLocale);
        result.push(localeItem);

        result.push({ code: LOGOUT, label: "main_menu.logout", subItems: [] });
        return result;
    }

}
