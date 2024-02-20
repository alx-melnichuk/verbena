import { ChangeDetectionStrategy, Component, ElementRef, HostBinding, Input, Renderer2, ViewEncapsulation } from '@angular/core';
import { CommonModule } from '@angular/common';
import { Router, RouterLink, RouterOutlet } from '@angular/router';
import { TranslateModule, TranslateService } from '@ngx-translate/core';

import { FooterComponent } from './components/footer/footer.component';
import { HeaderComponent } from './components/header/header.component';
import { UserService } from './entities/user/user.service';
import { AUTHORIZATION_DENIED, ROUTE_LOGIN, ROUTE_VIEW } from './common/routes';
import { InitializationService } from './common/initialization.service';

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

  constructor(
    public translate: TranslateService,
    public userService: UserService,
    public renderer: Renderer2,
    private router: Router,
    private initializationService: InitializationService,
  ) {
    this.initializationService.setDarkTheme(false, this.renderer);
  }

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

  // ** Private API **

}
