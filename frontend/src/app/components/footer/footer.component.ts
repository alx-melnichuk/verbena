import { ChangeDetectionStrategy, Component, HostBinding, Input, ViewEncapsulation } from '@angular/core';
import { CommonModule } from '@angular/common';
import { RouterLink } from '@angular/router';
import { TranslateModule } from '@ngx-translate/core';

import { ROUTE_LOGIN } from 'src/app/common/routes';
import { ProfileDto } from 'src/app/lib-profile/profile-api.interface';

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
  public profileDto: ProfileDto | null = null;
  
  @HostBinding('class.ft-is-profile')
  get isProfileDto(): boolean {
    return !!this.profileDto;
  }

  linkLogin = ROUTE_LOGIN;
  
  constructor() {
  }

  // ** Public API **

}
