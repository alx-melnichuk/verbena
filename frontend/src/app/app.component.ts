import { ChangeDetectionStrategy, Component, HostListener, OnInit, Renderer2, ViewEncapsulation } from '@angular/core';
import { CommonModule } from '@angular/common';
import { Router, RouterLink, RouterOutlet } from '@angular/router';
import { TranslateService } from '@ngx-translate/core';

import { COLOR_SCHEME_LIST } from './common/constants';
import { InitializationService } from './common/initialization.service';
import { AUTHORIZATION_DENIED, ROUTE_LOGIN } from './common/routes';
import { FooterComponent } from './components/footer/footer.component';
import { HeaderComponent, HM_LOGOUT, HM_SET_COLOR_SCHEME, HM_SET_LOCALE } from './components/header/header.component';
import { ACCESS_TOKEN, ProfileService } from './lib-profile/profile.service';
import { LocaleService } from './common/locale.service';

@Component({
    selector: 'app-root',
    standalone: true,
    imports: [CommonModule, RouterLink, RouterOutlet, HeaderComponent, FooterComponent],
    templateUrl: './app.component.html',
    styleUrls: ['./app.component.scss', 'app-panel-colors.component.scss', 'app-screen-size.component.scss'],
    encapsulation: ViewEncapsulation.None,
    changeDetection: ChangeDetectionStrategy.OnPush,
})
export class AppComponent implements OnInit {
    public title = 'verbena';
    public linkLogin = ROUTE_LOGIN;
    public locale: string | null = null;
    public colorScheme: string | null = null;

    @HostListener('window:storage', ['$event'])
    public windowStorage(event: StorageEvent): void {
        // Check for the presence of an authorization token.
        if (event.key == ACCESS_TOKEN && !this.profileService.hasAccessTokenInLocalStorage()) {
            // If there is no authorization token in the storage, then the current session is closed.
            // Clear the authorization token value.
            this.profileService.setProfileDto();
            this.profileService.setUserTokensDto();
            // And you need to go to the "login" tab.
            window.setTimeout(() => this.router.navigateByUrl(ROUTE_LOGIN, { replaceUrl: true }), 0);
        }
    }

    public get currentRoute(): string {
        return window.location.pathname;
    }
    public set currentRoute(value: string) {
    }

    constructor(
        public translate: TranslateService,
        public profileService: ProfileService,
        public renderer: Renderer2,
        private router: Router,
        private localeService: LocaleService,
        private initializationService: InitializationService,
    ) {
        const theme = this.profileService.profileDto?.theme || COLOR_SCHEME_LIST[0];
        this.initializationService.setColorScheme(theme, this.renderer);
    }

    async ngOnInit(): Promise<void> {
        this.locale = this.localeService.getLocale();
        this.colorScheme = this.initializationService.getColorScheme();
    }

    // ** Public API **

    public doCommand(event: Record<string, string>): void {
        const key = Object.keys(event)[0];
        const value = event[key];
        switch (key) {
            case HM_LOGOUT: this.doLogout(); break;
            case HM_SET_LOCALE: this.doSetLocale(value); break;
            case HM_SET_COLOR_SCHEME: this.doSetColorScheme(value); break;
        }
    }

    // ** Private API **

    private async doLogout(): Promise<void> {
        await this.profileService.logout();
        let currentRoute = window.location.pathname;
        const idx = AUTHORIZATION_DENIED.findIndex((item) => currentRoute.startsWith(item));
        currentRoute = (idx > -1 ? currentRoute : ROUTE_LOGIN);
        const queryParams = this.router.routerState.snapshot.root.queryParams;

        return new Promise(resolve =>
            window.setTimeout(() => {
                return this.router.navigate([currentRoute], { queryParams }) // ByUrl(currentRoute, { });
                    .finally(() => resolve);
            }, 0)
        );
    }

    private doSetLocale(value: string): void {
        this.localeService.setLocale(value)
            .finally(() => {
                this.locale = this.localeService.getLocale();
            });
    }

    private doSetColorScheme(value: string): void {
        this.initializationService.setColorScheme(value, this.renderer);
        this.colorScheme = this.initializationService.getColorScheme();
    }

}
