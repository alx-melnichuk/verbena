import { ChangeDetectionStrategy, Component, HostListener, Renderer2, ViewEncapsulation } from '@angular/core';
import { CommonModule } from '@angular/common';
import { Router, RouterLink, RouterOutlet } from '@angular/router';
import { TranslateModule, TranslateService } from '@ngx-translate/core';

import { FooterComponent } from './components/footer/footer.component';
import { HeaderComponent } from './components/header/header.component';
import { AUTHORIZATION_DENIED, ROUTE_LOGIN } from './common/routes';
import { InitializationService } from './common/initialization.service';
import { ACCESS_TOKEN, ProfileService } from './lib-profile/profile.service';

@Component({
  selector: 'app-root',
  standalone: true,
  imports: [CommonModule, RouterLink, RouterOutlet, TranslateModule, HeaderComponent, FooterComponent],
  templateUrl: './app.component.html',
  styleUrls: ['./app.component.scss'],
  encapsulation: ViewEncapsulation.None,
  changeDetection: ChangeDetectionStrategy.OnPush,
})
export class AppComponent {
  public title = 'verbena';
  public linkLogin = ROUTE_LOGIN;

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
    this.initializationService.setDarkTheme(false, this.renderer);
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

  // ** Private API **

}
