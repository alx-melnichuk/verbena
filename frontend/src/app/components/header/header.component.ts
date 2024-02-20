import { CommonModule } from '@angular/common';
import { ChangeDetectionStrategy, Component, EventEmitter, HostBinding, Input, Output, Renderer2, ViewEncapsulation } from '@angular/core';
import { RouterLink } from '@angular/router';
import { TranslateModule } from '@ngx-translate/core';

import { UserDto } from 'src/app/entities/user/user-dto';
import { MainMenuComponent } from '../main-menu/main-menu.component';
import { InitializationService } from 'src/app/common/initialization.service';

@Component({
  selector: 'app-header',
  standalone: true,
  imports: [CommonModule, RouterLink, TranslateModule, MainMenuComponent],
  templateUrl: './header.component.html',
  styleUrls: ['./header.component.scss'],
  encapsulation: ViewEncapsulation.None,
  changeDetection: ChangeDetectionStrategy.OnPush,
})
export class HeaderComponent {
  @Input()
  public userInfo: UserDto | null = null;
  @Output()
  readonly logout: EventEmitter<void> = new EventEmitter();

  public linkDashboard: string = 'login';

  @HostBinding('class.hd-user-info')
  get isUserInfo(): boolean {
    return !!this.userInfo;
  }

  constructor(public renderer: Renderer2, public initializationService: InitializationService) {
  }

  // ** Public API **

  public doSetDarkTheme(value: boolean): void {
    this.initializationService.setDarkTheme(value, this.renderer);
  }

  public doLogout(): void {
    this.logout.emit();
  }
}
