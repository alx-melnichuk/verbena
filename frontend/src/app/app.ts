import { CommonModule } from "@angular/common";
import {
    Component, ViewEncapsulation, ChangeDetectionStrategy, ChangeDetectorRef, inject, signal, HostListener,
} from "@angular/core";
import { RouterOutlet, Router } from "@angular/router";
import { environment as env } from "../environments/environment";
import { AppearanceSvr } from "./common/appearance-svr";
import { LocaleSrv } from "./common/locale-srv";
import { ROUTE_LOGIN, AUTHENT_DENIED } from "./common/routes";
import { SessionSrv, ACCESS_TOKEN } from "./common/session-srv";
import { ThemeSvr } from "./common/theme-svr";
import { Footer } from "./components/footer/footer";
import { Header, HM_LOGOUT, HM_SET_LOCALE, HM_SET_COLOR_SCHEME } from "./components/header/header";
import { UserSrv } from "./lib-user/user-srv";

@Component({
    selector: "app-root",
    standalone: true,
    imports: [CommonModule, RouterOutlet, Header, Footer],
    templateUrl: "./app.html",
    styleUrl: "./app.scss",
    encapsulation: ViewEncapsulation.None,
    changeDetection: ChangeDetectionStrategy.OnPush,
})
export class App {
    private appearanceSvr: AppearanceSvr = inject(AppearanceSvr);
    private changeDetector: ChangeDetectorRef = inject(ChangeDetectorRef);
    public localeSrv: LocaleSrv = inject(LocaleSrv);
    private router: Router = inject(Router);
    public sessionSrv: SessionSrv = inject(SessionSrv);
    public themeSvr: ThemeSvr = inject(ThemeSvr);
    private userSrv: UserSrv = inject(UserSrv);

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
        if (env.logLevel & 4) { console.info(`App(); locale.getLocale(): ${this.localeSrv.getLocale()}`); }
        // #const user = this.sessionSrv.getUser();
    }

    // ** Public API **

    public doCommand(event: Record<string, string>): void {
        const key = Object.keys(event)[0];
        const value = event[key];
        switch (key) {
            case HM_LOGOUT: this.doLogout(); break;
            case HM_SET_LOCALE: this.doSetLocale(value); break;
            case HM_SET_COLOR_SCHEME: this.doSetTheme(value); break;
        }
    }

    public doSetLocale(value: string): void {
        this.localeSrv.setLocale(value)
            .then((response: boolean) => {
                if (!!response) {
                    this.localeSrv.saveToStorage();
                }
            });
    }

    public doSetTheme(value: string): void {
        this.themeSvr.setTheme(value);
        this.themeSvr.saveToStorage();
    }

    public doLogout(): void {
        let currRoute = window.location.pathname;
        const idx = AUTHENT_DENIED.findIndex((item) => currRoute.startsWith(item));
        currRoute = (idx > -1 ? currRoute : ROUTE_LOGIN);
        const queryParams = (idx > -1 ? this.router.routerState.snapshot.root.queryParams : {});
        this.userSrv.logout()
            .then(() => {
                this.sessionSrv.removeUser();
                this.sessionSrv.removeUserTokens();

                const locale = this.localeSrv.getFromStorage();
                if (env.logLevel & 4) { console.info(`App.doLogout() localeSrv.setLocale(${locale});`); }
                this.localeSrv.setLocale(locale);

                const settings = this.appearanceSvr.getFromStorage();
                if (env.logLevel & 4) { console.info(`App.doLogout() appearanceSvr.setSettings((${JSON.stringify(settings)});`); }
                this.appearanceSvr.setSettings(settings);

                const theme = this.themeSvr.getFromStorage();
                if (env.logLevel & 4) { console.info(`App.doLogout() themeSvr.setTheme((${theme});`); }
                this.themeSvr.setTheme(theme);

                Promise.resolve().then(() => {
                    this.router.navigate([currRoute], { queryParams, onSameUrlNavigation: "reload" })
                        .finally(() => this.changeDetector.markForCheck());
                });
            });
    }

    // ** Private API **

}
