import { CommonModule, NgTemplateOutlet } from "@angular/common";
import {
    Component, ViewEncapsulation, ChangeDetectionStrategy, ChangeDetectorRef, inject, Renderer2, signal, HostListener,
    TemplateRef, ViewChild, ViewContainerRef,
} from "@angular/core";
import { RouterOutlet, Router } from "@angular/router";
import { environment } from "../environments/environment";
import { ColorSchemeSrv } from "./common/color-scheme-srv";
import { LocaleSrv } from "./common/locale-srv";
import { ROUTE_LOGIN, AUTHENT_DENIED } from "./common/routes";
import { Footer } from "./components/footer/footer";
import { Header, HM_LOGOUT, HM_SET_LOCALE, HM_SET_COLOR_SCHEME } from "./components/header/header";
import { ACCESS_TOKEN, SessionSrv } from "./common/session-srv";
import { UserSrv } from "./lib-user/user-srv";

@Component({
    selector: "app-root",
    standalone: true,
    imports: [CommonModule, NgTemplateOutlet, RouterOutlet, Header, Footer],
    templateUrl: "./app.html",
    styleUrl: "./app.scss",
    encapsulation: ViewEncapsulation.None,
    changeDetection: ChangeDetectionStrategy.OnPush,
})
export class App {
    private changeDetector: ChangeDetectorRef = inject(ChangeDetectorRef);
    public colorSchemeSrv: ColorSchemeSrv = inject(ColorSchemeSrv);
    public localeSrv: LocaleSrv = inject(LocaleSrv);
    private renderer: Renderer2 = inject(Renderer2);
    private router: Router = inject(Router);
    public sessionSrv: SessionSrv = inject(SessionSrv);
    private userSrv: UserSrv = inject(UserSrv);

    @ViewChild("outlet", { read: ViewContainerRef })
    public outletRef!: ViewContainerRef;
    @ViewChild("content", { read: TemplateRef })
    public contentRef!: TemplateRef<unknown>;

    protected readonly title = signal("verbena");

    public get currentRoute(): string { return window.location.pathname; }
    public set currentRoute(value: string) { }

    @HostListener("window:storage", ["$event"])
    public windowStorage(event: StorageEvent): void {
        // Check for the presence of an authorization token.
        if (event.key == ACCESS_TOKEN && !!this.sessionSrv.getAccessToken()) {
            // If there is no authorization token in the storage, then the current session is closed.
            // Clear the authorization token value.
            this.sessionSrv.removeUserTokens();
            this.sessionSrv.removeUser();
            // And you need to go to the "login" tab.
            Promise.resolve().then(() => {
                this.router.navigateByUrl(ROUTE_LOGIN, { replaceUrl: true });
            });
        }
    }

    constructor() {
        if (environment.logLevel > 0) { console.info(`App(); locale.getLocale(): ${this.localeSrv.getLocale()}`); }
        const user = this.sessionSrv.getUser();
        this.localeSrv.setLocale(user?.locale || this.localeSrv.getFromLocalStorage());
        this.setHtmlLangAttribute(this.localeSrv.getLocale().slice(0, 2));
        this.colorSchemeSrv.setSchemeLightName(user?.theme || this.colorSchemeSrv.getFromLocalStorage(), this.renderer);
    }

    // ** Public API **

    public doCommand(event: Record<string, string>): void {
        const key = Object.keys(event)[0];
        const value = event[key];
        switch (key) {
            case HM_LOGOUT: this.doLogout(); break;
            case HM_SET_LOCALE: this.doSetLocale(value); break;
            case HM_SET_COLOR_SCHEME: this.colorSchemeSrv.setSchemeLightName(value, this.renderer); break;
        }
    }

    // ** Private API **

    private doSetLocale(value: string): void {
        this.localeSrv.setLocale(value)
            .then((response: boolean) => {
                if (!!response) {
                    this.setHtmlLangAttribute(this.localeSrv.getLocale().slice(0, 2));
                }
            });
    }

    private doLogout(): void {
        let currRoute = window.location.pathname;
        const idx = AUTHENT_DENIED.findIndex((item) => currRoute.startsWith(item));
        currRoute = (idx > -1 ? currRoute : ROUTE_LOGIN);
        const queryParams = (idx > -1 ? this.router.routerState.snapshot.root.queryParams : {});
        this.userSrv.logout()
            .then(() => {
                this.sessionSrv.removeUser();
                this.sessionSrv.removeUserTokens();
                Promise.resolve().then(() => {
                    this.router.navigate([currRoute], { queryParams, onSameUrlNavigation: "reload" })
                        .finally(() => this.changeDetector.markForCheck());
                });
            });
    }

    private rerender() {
        this.outletRef.clear();
        this.outletRef.createEmbeddedView(this.contentRef);
    }

    private setHtmlLangAttribute(lang: string): void {
        if (!!lang && typeof document !== "undefined") {
            document.documentElement.setAttribute("lang", lang);
        }
    }
}
