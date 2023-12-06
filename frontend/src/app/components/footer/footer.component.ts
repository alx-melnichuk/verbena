import { ChangeDetectionStrategy, Component, HostBinding, Input, ViewEncapsulation } from '@angular/core';
import { CommonModule } from '@angular/common';
import { RouterLink } from '@angular/router';
import { TranslateModule } from '@ngx-translate/core';

import { UserDto } from 'src/app/entities/user/user-dto';
import { ROUTE_LOGIN } from 'src/app/common/routes';

@Component({
  selector: 'app-footer',
  standalone: true,
  imports: [CommonModule, RouterLink, TranslateModule],
  templateUrl: './footer.component.html',
  styleUrls: ['./footer.component.scss'],
  encapsulation: ViewEncapsulation.None,
  changeDetection: ChangeDetectionStrategy.OnPush
})
export class FooterComponent {
  @Input()
  public userInfo: UserDto | null = null;
  
  @HostBinding('class.ft-user-info')
  get isUserInfo(): boolean {
    return !!this.userInfo;
  }

  linkLogin = ROUTE_LOGIN;
  
  constructor() {
  }

  // ** Public API **

}
