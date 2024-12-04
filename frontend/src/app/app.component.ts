import { ChangeDetectionStrategy, Component, HostListener, OnInit, Renderer2, ViewEncapsulation } from '@angular/core';
import { CommonModule } from '@angular/common';
import { Router, RouterLink, RouterOutlet } from '@angular/router';
import { TranslateModule, TranslateService } from '@ngx-translate/core';

import { THEME_LIST } from './common/constants';
import { InitializationService } from './common/initialization.service';
import { AUTHORIZATION_DENIED, ROUTE_LOGIN } from './common/routes';
import { FooterComponent } from './components/footer/footer.component';
import { HeaderComponent } from './components/header/header.component';
import { ACCESS_TOKEN, ProfileService } from './lib-profile/profile.service';

@Component({
  selector: 'app-root',
  standalone: true,
  imports: [CommonModule, RouterLink, RouterOutlet, TranslateModule, HeaderComponent, FooterComponent],
  templateUrl: './app.component.html',
  styleUrl: './app.component.scss',
  encapsulation: ViewEncapsulation.None,
  changeDetection: ChangeDetectionStrategy.OnPush,
})
export class AppComponent implements OnInit {
  public title = 'verbena';
  public linkLogin = ROUTE_LOGIN;
  public locale: string | null = null;
  public theme: string | null = null;

  @HostListener('window:storage', ['$event'])
  public windowStorage(event: StorageEvent): void {
    // Check for the presence of an authorization token.
    if (event.key == ACCESS_TOKEN && !this.profileService.hasAccessTokenInLocalStorage()) {
      // If there is no authorization token in the storage, then the current session is closed.
      // Clear the authorization token value.
      this.profileService.setProfileDto();
      this.profileService.setProfileTokensDto();
      // And you need to go to the "login" tab.
      this.router.navigateByUrl(ROUTE_LOGIN, { replaceUrl: true });
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
    private initializationService: InitializationService,
  ) {
    const theme = this.profileService.profileDto?.theme || THEME_LIST[0];
    this.initializationService.setTheme(theme, this.renderer);
  }

  async ngOnInit(): Promise<void> {
    this.locale = this.initializationService.getLocale();
    this.theme = this.initializationService.getTheme();
  }

  // ** Public API **

  public async doLogout(): Promise<void> {
    await this.profileService.logout();
    let currentRoute = window.location.pathname;
    const idx = AUTHORIZATION_DENIED.findIndex((item) => currentRoute.startsWith(item));
    currentRoute = (idx > -1 ? currentRoute : ROUTE_LOGIN);
    const queryParams = this.router.routerState.snapshot.root.queryParams;
    await this.router.navigate([currentRoute], { queryParams }); // ByUrl(currentRoute, { });
    return Promise.resolve();
  }

  public doSetLocale(value: string): void {
    this.initializationService.setLocale(value)
      .finally(() => {
        this.locale = this.initializationService.getLocale();
      });
  }

  public doSetTheme(value: string): void {
    this.initializationService.setTheme(value, this.renderer);
    this.theme = this.initializationService.getTheme();
  }

  // ** Private API **

}
