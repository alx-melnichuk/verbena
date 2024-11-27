import { CommonModule } from '@angular/common';
import {
  ChangeDetectionStrategy, Component, EventEmitter, HostBinding, Input, OnChanges, Output, Renderer2, SimpleChanges, 
  ViewEncapsulation
} from '@angular/core';
import { RouterLink, RouterLinkActive } from '@angular/router';
import { TranslateModule } from '@ngx-translate/core';

import { InitializationService } from 'src/app/common/initialization.service';
import { MainMenu, MainMenuUtil } from 'src/app/common/main-menu';
import { ROUTE_LOGIN } from 'src/app/common/routes';
import { ProfileDto } from 'src/app/lib-profile/profile-api.interface';

import { MainMenuComponent } from '../main-menu/main-menu.component';
import { THEME_DARK, THEME_LIGHT } from 'src/app/common/constants';

@Component({
  selector: 'app-header',
  exportAs: 'appHeader',
  standalone: true,
  imports: [CommonModule, RouterLink, RouterLinkActive, TranslateModule, MainMenuComponent],
  templateUrl: './header.component.html',
  styleUrl: './header.component.scss',
  encapsulation: ViewEncapsulation.None,
  changeDetection: ChangeDetectionStrategy.OnPush,
})
export class HeaderComponent implements OnChanges {
  @Input()
  public currentRoute: string | null = null;
  @Input()
  public locale: string | null = null;
  @Input()
  public profileDto: ProfileDto | null = null;
  @Input()
  public theme: string | null = null;

  @Output()
  readonly logout: EventEmitter<void> = new EventEmitter();
  @Output()
  readonly setLocale: EventEmitter<string> = new EventEmitter();
  @Output()
  readonly setTheme: EventEmitter<string> = new EventEmitter();

  public menuList: MainMenu[] = [];
  public linkLogin = ROUTE_LOGIN;

  @HostBinding('class.hd-is-authorized')
  get isAuthorizedVal(): boolean {
    return !!this.profileDto;
  }

  constructor(public renderer: Renderer2, public initializationService: InitializationService) {
  }

  ngOnChanges(changes: SimpleChanges): void {
    if (!!changes['profileDto'] || !!changes['currentRoute']) {
      this.menuList = MainMenuUtil.getList(this.currentRoute || '', this.profileDto != null);
    }
  }

  // ** Public API **

  public doLogout(): void {
    this.logout.emit();
  }

  public doSetLocale(value: string): void {
    this.setLocale.emit(value);
  }

  public doSetTheme(value: string): void {
    this.setTheme.emit(value);
  }

  // ** Private API **

}
