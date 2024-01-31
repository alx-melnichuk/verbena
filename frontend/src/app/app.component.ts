import { ChangeDetectionStrategy, Component, HostBinding, ViewEncapsulation } from '@angular/core';
import { CommonModule } from '@angular/common';
import { Router, RouterLink, RouterOutlet } from '@angular/router';
import { TranslateModule, TranslateService } from '@ngx-translate/core';

import { FooterComponent } from './components/footer/footer.component';
import { HeaderComponent } from './components/header/header.component';
import { UserService } from './entities/user/user.service';
import { AUTHORIZATION_DENIED, ROUTE_LOGIN, ROUTE_VIEW } from './common/routes';

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
  public isLightThemeVal: boolean = true;

  @HostBinding('class.light-theme')
  get isLightTheme(): boolean {
    return !!this.isLightThemeVal;
  }
  @HostBinding('class.dark-theme')
  get isDarkTheme(): boolean {
    return !this.isLightThemeVal;
  }

  constructor(public translate: TranslateService, private router: Router, public userService: UserService) {}

  // ** Public API **

  public async doLogout(): Promise<void> {
    await this.userService.logout();
    let currentRoute = window.location.pathname;
    const idx = AUTHORIZATION_DENIED.findIndex((item) => currentRoute.startsWith(item));
    currentRoute = (idx > -1 ? currentRoute : ROUTE_LOGIN);
    const queryParams = this.router.routerState.snapshot.root.queryParams;
    await this.router.navigate([currentRoute], { queryParams }); // ByUrl(currentRoute, { });
    return Promise.resolve();
  }
}
