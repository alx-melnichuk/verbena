import { ChangeDetectionStrategy, Component, ViewEncapsulation } from '@angular/core';
import { CommonModule } from '@angular/common';
import { RouterLink, RouterOutlet } from '@angular/router';
import { TranslateModule, TranslateService } from '@ngx-translate/core';

import { HeaderComponent } from './components/header/header.component';
import { UserService } from './entities/user/user.service';
import { ROUTE_LOGIN } from './common/routes';

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
  constructor(public translate: TranslateService, public userService: UserService) {}
}
