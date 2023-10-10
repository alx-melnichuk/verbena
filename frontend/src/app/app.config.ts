import { HttpClient, provideHttpClient } from '@angular/common/http';
import { APP_INITIALIZER, ApplicationConfig, importProvidersFrom } from '@angular/core';
import { provideAnimations } from '@angular/platform-browser/animations';
import { provideRouter } from '@angular/router';
import { MatDialogModule } from '@angular/material/dialog';
import { MatSnackBarModule } from '@angular/material/snack-bar';
import { TranslateHttpLoader } from '@ngx-translate/http-loader';
import { TranslateLoader, TranslateModule } from '@ngx-translate/core';

import { routes } from './app.routes';
import { InitTranslateService } from './common/init-translate.service';

// AoT requires an exported function for factories
export const HTTP_LOADER_FACTORY = (httpClient: HttpClient): TranslateHttpLoader => {
  console.log(`HTTP_LOADER_FACTORY()`); // #
  return new TranslateHttpLoader(httpClient, './assets/i18n/', '.json');
};

export const INITIALIZE_TRANSLATE_FACTORY = (initTranslateService: InitTranslateService): any => {
  return (): Promise<any> => initTranslateService.init();
};

export const appConfig: ApplicationConfig = {
  providers: [
    provideRouter(routes),
    provideAnimations(),
    provideHttpClient(),
    importProvidersFrom(
      TranslateModule.forRoot({
        loader: {
          provide: TranslateLoader,
          deps: [HttpClient],
          useFactory: HTTP_LOADER_FACTORY,
        },
      })
    ),
    importProvidersFrom(MatDialogModule, MatSnackBarModule),
    InitTranslateService,
    {
      provide: APP_INITIALIZER,
      deps: [InitTranslateService],
      useFactory: INITIALIZE_TRANSLATE_FACTORY,
      multi: true,
    },
  ],
};
