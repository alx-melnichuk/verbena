import { ChangeDetectionStrategy, Component, HostBinding, Input, ViewEncapsulation } from '@angular/core';
import { CommonModule } from '@angular/common';
import { RouterLink } from '@angular/router';
import { TranslateModule } from '@ngx-translate/core';

import { ROUTE_LOGIN } from 'src/app/common/routes';

@Component({
  selector: 'app-footer',
  exportAs: 'appFooter',
  standalone: true,
  imports: [CommonModule, RouterLink, TranslateModule],
  templateUrl: './footer.component.html',
  styleUrls: ['./footer.component.scss'],
  encapsulation: ViewEncapsulation.None,
  changeDetection: ChangeDetectionStrategy.OnPush
})
export class FooterComponent {
  @Input()
  public isAuthorized: boolean | null | undefined;
  
  @HostBinding('class.ft-is-authorized')
  get isAuthorizedVal(): boolean {
    return !!this.isAuthorized;
  }

  linkLogin = ROUTE_LOGIN;
  
  constructor() {
  }

  // ** Public API **

}
