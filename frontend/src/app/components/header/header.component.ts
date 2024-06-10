import { CommonModule } from '@angular/common';
import {
  ChangeDetectionStrategy, Component, EventEmitter, HostBinding, Input, OnChanges, Output, Renderer2, SimpleChanges, ViewEncapsulation
} from '@angular/core';
import { RouterLink, RouterLinkActive } from '@angular/router';
import { TranslateModule } from '@ngx-translate/core';

import { UserDto } from 'src/app/entities/user/user-dto';
import { MainMenuComponent } from '../main-menu/main-menu.component';
import { InitializationService } from 'src/app/common/initialization.service';
import { MainMenu, MainMenuUtil } from 'src/app/common/main-menu';
import { ROUTE_LOGIN } from 'src/app/common/routes';


@Component({
  selector: 'app-header',
  standalone: true,
  imports: [CommonModule, RouterLink, RouterLinkActive, TranslateModule, MainMenuComponent],
  templateUrl: './header.component.html',
  styleUrls: ['./header.component.scss'],
  encapsulation: ViewEncapsulation.None,
  changeDetection: ChangeDetectionStrategy.OnPush,
})
export class HeaderComponent implements OnChanges {
  @Input()
  public userInfo: UserDto | null = null;
  @Input()
  public currentRoute: string | null = null;
  @Output()
  readonly logout: EventEmitter<void> = new EventEmitter();

  public menuList: MainMenu[] = [];
  public linkLogin = ROUTE_LOGIN;

  @HostBinding('class.hd-user-info')
  get isUserInfo(): boolean {
    return !!this.userInfo;
  }

  constructor(public renderer: Renderer2, public initializationService: InitializationService) {
  }

  ngOnChanges(changes: SimpleChanges): void {
    if (!!changes['userInfo'] || !!changes['currentRoute']) {
      this.menuList = MainMenuUtil.getList(this.currentRoute || '', this.userInfo != null);
    }
  }

  // ** Public API **

  public setDarkTheme(value: boolean): void {
    this.initializationService.setDarkTheme(value, this.renderer);
  }

  public doLogout(): void {
    this.logout.emit();
  }

  // ** Private API **

}
