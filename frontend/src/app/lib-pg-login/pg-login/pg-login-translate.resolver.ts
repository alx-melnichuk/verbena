import { inject } from "@angular/core";
import { ActivatedRouteSnapshot, ResolveFn, RouterStateSnapshot } from "@angular/router";
import { InterpolatableTranslationObject, TranslateService } from "@ngx-translate/core";
import { Observable } from "rxjs";
import { LocaleSrv } from "../../common/locale-srv";
//#import { environment as env } from "../../../environments/environment";

export const pgLoginTranslateResolver: ResolveFn<Observable<InterpolatableTranslationObject>>
    = (_route: ActivatedRouteSnapshot, _state: RouterStateSnapshot) => {
        const locale = inject(LocaleSrv);
        const translate = inject(TranslateService);

        // Download translations before starting the сomponent.
        translate.addLangs(locale.localeList);
        translate.setDefaultLang(locale.localeDefault);

        // Get the current locale.
        const localeLanguage = locale.getLocale();
        // Get translations for the current locale from the core module.
        const translationObject = locale.translationsByLang(localeLanguage);
        if (!!translationObject) {
            // #if (env.logLevel & LOG_PG_PROFILE) { console.info(`pgLoginResolver() translate.setTranslation(${localeLanguage}); obj:`, { ...translationObject }); }
            // Add translations from the main module to this module.
            translate.setTranslation(localeLanguage, translationObject, true);
        }
        return translate.use(localeLanguage);
        /*#return new Promise<void>((resolve: () => void, reject: (reason: unknown) => void) => {
            translate.use(localeLanguage).pipe(first())
                .subscribe({
                    next: (data) => {
                        if (env.logLevel  & LOG_PG_PROFILE) { console.info(`pgSignupResolver() translate.use((${localeLanguage})...Ok data:`, data); }
                        resolve();
                    },
                    error: (err) => reject(err)
                });
        });*/
    };