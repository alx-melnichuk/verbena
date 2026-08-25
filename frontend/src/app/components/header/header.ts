import { CommonModule } from "@angular/common";
import {
    ChangeDetectionStrategy, ChangeDetectorRef, Component, EventEmitter, HostBinding, HostListener, inject, Input, OnChanges,
    Output, SimpleChanges, ViewEncapsulation
} from "@angular/core";
import { FormsModule } from "@angular/forms";
import { MatButtonModule } from "@angular/material/button";
import { MatMenuModule } from "@angular/material/menu";
import { MatSlideToggleModule } from "@angular/material/slide-toggle";
import { MatToolbarModule } from "@angular/material/toolbar";
import { RouterLink, RouterLinkActive } from "@angular/router";
import { TranslatePipe } from "@ngx-translate/core";
import { COLOR_SCHEME_LIST } from "../../common/color-scheme-srv";
import { LOCALE_LIST } from "../../common/locale-srv";
import { MainMenu, MainMenuUtil } from "../../common/main-menu";
import { MAIN_MENU_LIST } from "../../common/routes";
import { User } from "../../common/session-srv";
import { ReplaceWithZeroUtil } from "../../utils/replace-with-zero.util";
import { StringUtil } from "../../utils/string.util";
import { Image } from "../image/image";

export const HM_LOGOUT = "logout";
export const HM_SET_LOCALE = "setLocale";
export const HM_SET_COLOR_SCHEME = "setColorScheme";

const CN_MIN_WINDOW_WIDTH = 850; // Minimum window width for displaying the main menu.
const CN_ResizeEventTimeout = 150; // milliseconds

@Component({
    selector: "app-header",
    exportAs: "appHeader",
    standalone: true,
    imports: [CommonModule, FormsModule, RouterLink, RouterLinkActive, MatButtonModule, MatMenuModule,
        MatSlideToggleModule, MatToolbarModule, TranslatePipe, Image],
    templateUrl: "./header.html",
    styleUrl: "./header.scss",
    encapsulation: ViewEncapsulation.None,
    changeDetection: ChangeDetectionStrategy.OnPush,
})
export class Header implements OnChanges {
    @Input()
    public currentRoute: string | null = null;
    @Input()
    public locale: string | null = null;
    @Input()
    public user: User | null = null;
    @Input()
    public currentScheme: string | null = null;

    @Output()
    readonly command: EventEmitter<Record<string, string>> = new EventEmitter();

    private changeDetector: ChangeDetectorRef = inject(ChangeDetectorRef);

    public colorSchemeList: string[] = [...COLOR_SCHEME_LIST];
    public isShowTheme: boolean = true;
    public isShowLocale: boolean = true;
    public isShowLogout: boolean = true;
    public nickname: string = "";
    public mainMenuItems: MainMenu[] = [];
    public localeList: string[] = [...LOCALE_LIST];
    public panelMenuItems: MainMenu[] = [];
    public userNameSymbols: string = "";

    @HostBinding("class.h-is-authorized")
    get isAuthorizedVal(): boolean {
        return !!this.user;
    }

    private timerResizeEvent: any = null;
    @HostListener("window:resize", ["$event"])
    public doScrollPanel(event: Event): void {
        event.preventDefault();
        event.stopPropagation();

        if (this.timerResizeEvent !== null) {
            clearTimeout(this.timerResizeEvent);
        }
        this.timerResizeEvent = setTimeout(() => {
            this.timerResizeEvent = null;

            this.prepareMenuItems(this.user, MAIN_MENU_LIST, CN_MIN_WINDOW_WIDTH < this.getWidth());
            this.changeDetector.markForCheck();
        }, CN_ResizeEventTimeout);
    }


    ngOnChanges(changes: SimpleChanges): void {
        if (!!changes["user"] || !!changes["currentRoute"]) {
            this.prepareMenuItems(this.user, MAIN_MENU_LIST, CN_MIN_WINDOW_WIDTH < this.getWidth());
        }
        if (!!changes["user"]) {
            this.userNameSymbols = (StringUtil.capitalizeOnlyFirstLetter(this.user?.nickname) || "").slice(0, 2);
        }
    }

    // ** Public API **

    public doLogout(): void {
        this.doCommand(HM_LOGOUT, "");
    }

    public doSetLocale(value: string): void {
        this.doCommand(HM_SET_LOCALE, value);
    }

    public doSetColorScheme(value: string): void {
        this.doCommand(HM_SET_COLOR_SCHEME, value);
    }

    public prepareMenuItems(user: User | null, mainMenuList: string[], isShowMainMenu: boolean): void {
        this.nickname = ReplaceWithZeroUtil.replace(user?.nickname)
        const items = MainMenuUtil.getList(user != null, mainMenuList);
        this.mainMenuItems = isShowMainMenu ? items : [];
        this.panelMenuItems = isShowMainMenu ? [] : items;
    }

    // ** Private API **

    private getWidth(): number {
        return window.innerWidth || document.documentElement.clientWidth || document.body.clientWidth;
    }

    private doCommand(commandName: string, commandValue: string): void {
        this.command.emit({ [commandName]: commandValue });
    }
}
