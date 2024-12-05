import { CommonModule } from '@angular/common';
import {
  ChangeDetectionStrategy, Component, EventEmitter, HostBinding, Input, OnChanges, Output, Renderer2, SimpleChanges, 
  ViewEncapsulation
} from '@angular/core';
import { FormsModule } from '@angular/forms';
import { RouterLink, RouterLinkActive } from '@angular/router';
import { MatButtonModule } from '@angular/material/button';
import { MatMenuModule } from '@angular/material/menu';
import { MatSlideToggleModule } from '@angular/material/slide-toggle';
import { MatToolbarModule } from '@angular/material/toolbar';
import { TranslateModule } from '@ngx-translate/core';

import { InitializationService } from 'src/app/common/initialization.service';
import { MainMenu, MainMenuUtil } from 'src/app/common/main-menu';
import { LOCALE_LIST, THEME_LIST } from 'src/app/common/constants';
import { ROUTE_LOGIN, ROUTE_STREAM_CREATE, ROUTE_STREAM_LIST } from 'src/app/common/routes';
import { ProfileDto } from 'src/app/lib-profile/profile-api.interface';

import { MainMenuComponent } from '../main-menu/main-menu.component';

@Component({
  selector: 'app-header',
  exportAs: 'appHeader',
  standalone: true,
  imports: [CommonModule, FormsModule, RouterLink, RouterLinkActive,
    MatButtonModule, MatMenuModule, MatSlideToggleModule, MatToolbarModule,  
    TranslateModule, MainMenuComponent],
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

  public pageMenuList: MainMenu[] = [];
  public localeList: string[] = ['', ...LOCALE_LIST];
  public themeList: string[] = ['', ...THEME_LIST];

  public linkLogin = ROUTE_LOGIN;
  public linkMyStreams = ROUTE_STREAM_LIST;
  public linkCreateStream = ROUTE_STREAM_CREATE;
  public isDarkTheme: boolean = false;

  public isShowMyStreams: boolean = true;
  public isShowCreateStream: boolean = true;
  public isShowTheme: boolean = true;
  public isShowLocale: boolean = true;
  public isShowLogout: boolean = true;

  @HostBinding('class.hd-is-authorized')
  get isAuthorizedVal(): boolean {
    return !!this.profileDto;
  }

  constructor(public renderer: Renderer2, public initializationService: InitializationService) {
  }

  ngOnChanges(changes: SimpleChanges): void {
    if (!!changes['profileDto'] || !!changes['currentRoute']) {
      this.pageMenuList = MainMenuUtil.getList(this.profileDto != null);
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
