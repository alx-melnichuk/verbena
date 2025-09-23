import { DOCUMENT } from '@angular/common';
import { Inject, Injectable, Renderer2 } from '@angular/core';
import { Router } from '@angular/router';
import { TranslateService } from '@ngx-translate/core';

import { ProfileService } from '../lib-profile/profile.service';
import { AuthorizationUtil } from '../utils/authorization.util';

import { COLOR_SCHEME_LIST, ENV_IS_PROD, LOCALE_EN, LOCALE_LIST, SCHEME_DARK, SCHEME_LIGHT } from './constants';
import { LOCALE, LocaleService } from './locale.service';
import { ROUTE_LOGIN } from './routes';

const COLOR_SCHEME = 'color-scheme';

@Injectable({
    providedIn: 'root',
})
export class InitializationService {
    private currColorScheme: string | null = null;
    private currLocale: string | null = null;

    get theme(): string | null {
        return this.currColorScheme;
    }
    set theme(value: string | null) {
    }

    constructor(
        @Inject(DOCUMENT) private document: Document,
        private router: Router,
        private translate: TranslateService,
        private localeService: LocaleService,
        private profileService: ProfileService,
    ) {
        if (!ENV_IS_PROD) { console.log(`#2-InitializationService();`); }
    }

    public initTranslate(): Promise<void | unknown> {
        // Download translations before starting the application.
        this.translate.addLangs(LOCALE_LIST);
        this.translate.setDefaultLang(LOCALE_EN);


        let locale: string | null = this.currLocale || window.localStorage.getItem(LOCALE);
        if (!!locale) {
            locale = this.localeService.findLocale(LOCALE_LIST, locale);
        }
        if (!locale) {
            const languages = this.getBrowserLanguages();
            for (let index = 0; index < languages.length && !locale; index++) {
                locale = this.localeService.findLocale(LOCALE_LIST, languages[index]);
            }
        }
        locale = locale || LOCALE_EN;
        console.log(`#initTranslate() localeService.setLocale(${locale});`); // #
        return this.localeService.setLocale(locale);
    }

    public async initSession(): Promise<void> {
        const isAuthorizationRequired = AuthorizationUtil.isAuthorizationRequired(window.location.pathname);
        const isAuthorizationDenied = AuthorizationUtil.isAuthorizationDenied(window.location.pathname);
        const isNotAuthorizationDenied = !isAuthorizationDenied;

        const isExistAccessToken = this.profileService.hasAccessTokenInLocalStorage();
        if (isAuthorizationRequired || (isNotAuthorizationDenied && isExistAccessToken)) {
            try {
                await this.profileService.getCurrentProfile();
                const locale = this.profileService.profileDto?.locale;
                if (!!locale && this.currLocale != locale) {
                    await this.localeService.setLocale(locale);
                }
            } catch {
                window.setTimeout(() => this.router.navigateByUrl(ROUTE_LOGIN, { replaceUrl: true }), 0);
            }
        }
        return Promise.resolve();
    }

    // ** Theme **
    public getColorScheme(): string | null {
        return this.currColorScheme;
    }

    public setColorScheme(value: string | null | undefined, renderer: Renderer2): void {
        const index = COLOR_SCHEME_LIST.indexOf(value || '');
        const theme = COLOR_SCHEME_LIST[index > -1 ? index : 0];
        if (this.currColorScheme != theme) {
            if (!!this.currColorScheme) {
                this.document.documentElement.style.setProperty(COLOR_SCHEME, null);
                this.document.documentElement.style.setProperty('--' + COLOR_SCHEME, null);
                renderer.removeClass(this.document.documentElement, this.currColorScheme);
                const scheme = this.currColorScheme.split('-')[0];
                if ([SCHEME_LIGHT, SCHEME_DARK].includes(scheme)) {
                    renderer.removeClass(this.document.documentElement, scheme);
                }
            }
            this.currColorScheme = theme;
            renderer.addClass(this.document.documentElement, theme);
            const scheme = this.currColorScheme.split('-')[0];
            if ([SCHEME_LIGHT, SCHEME_DARK].includes(scheme)) {
                this.document.documentElement.style.setProperty(COLOR_SCHEME, scheme);
                this.document.documentElement.style.setProperty('--' + COLOR_SCHEME, scheme);
                renderer.addClass(this.document.documentElement, scheme);
            }
        }
    }

    // ** Private Api **

    private getBrowserLanguages(): string[] {
        return (window.navigator as any).languages || [];
    }
}
