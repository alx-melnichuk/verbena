import { ChangeDetectionStrategy, Component, ViewEncapsulation } from '@angular/core';
import { CommonModule } from '@angular/common';
import { Router, RouterLink, RouterOutlet } from '@angular/router';
import { TranslateModule, TranslateService } from '@ngx-translate/core';

import { HeaderComponent } from './components/header/header.component';
import { UserService } from './entities/user/user.service';
import { AUTHORIZATION_DENIED, ROUTE_LOGIN, ROUTE_VIEW } from './common/routes';

@Component({
  selector: 'app-root',
  standalone: true,
  imports: [CommonModule, RouterLink, RouterOutlet, TranslateModule, HeaderComponent],
  templateUrl: './app.component.html',
  styleUrls: ['./app.component.scss'],
  encapsulation: ViewEncapsulation.None,
  changeDetection: ChangeDetectionStrategy.OnPush,
})
export class AppComponent {
  title = 'verbena';
  linkLogin = ROUTE_LOGIN;
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
