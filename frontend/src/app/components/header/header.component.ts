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
import { LOCALE_LIST, COLOR_SCHEME_LIST } from 'src/app/common/constants';
import { MAIN_MENU_LIST, ROUTE_LOGIN, ROUTE_STREAM_CREATE, ROUTE_STREAM_LIST } from 'src/app/common/routes';
import { ProfileDto } from 'src/app/lib-profile/profile-api.interface';

export const HM_LOGOUT = 'logout';
export const HM_SET_LOCALE = 'setLocale';
export const HM_SET_COLOR_SCHEME = 'setColorScheme';

@Component({
  selector: 'app-header',
  exportAs: 'appHeader',
  standalone: true,
  imports: [CommonModule, FormsModule, RouterLink, RouterLinkActive,
    MatButtonModule, MatMenuModule, MatSlideToggleModule, MatToolbarModule, 
    TranslateModule],
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
  public colorScheme: string | null = null;

  @Output()
  readonly command: EventEmitter<Record<string, string>> = new EventEmitter();

  public pageMenuList: MainMenu[] = [];
  public localeList: string[] = [...LOCALE_LIST];
  public colorSchemeList: string[] = [...COLOR_SCHEME_LIST];
  
  public linkLogin = ROUTE_LOGIN;
  public linkMyStreams = ROUTE_STREAM_LIST;
  public linkCreateStream = ROUTE_STREAM_CREATE;

  public isShowMyStreams: boolean = true;
  public isShowCreateStream: boolean = true;
  public isShowTheme: boolean = true;
  public isShowLocale: boolean = true;
  public isShowLogout: boolean = true;

  @HostBinding('class.h-is-authorized')
  get isAuthorizedVal(): boolean {
    return !!this.profileDto;
  }

  constructor(public renderer: Renderer2, public initializationService: InitializationService) {
  }

  ngOnChanges(changes: SimpleChanges): void {
    if (!!changes['profileDto'] || !!changes['currentRoute']) {
      this.pageMenuList = MainMenuUtil.getList(this.profileDto != null, MAIN_MENU_LIST);
    }
  }

  // ** Public API **

  public doLogout(): void {
    this.doCommand(HM_LOGOUT, '');
  }

  public doSetLocale(value: string): void {
    this.doCommand(HM_SET_LOCALE, value);
  }

  public doSetColorScheme(value: string): void {
    this.doCommand(HM_SET_COLOR_SCHEME, value);
  }

  // ** Private API **

  private doCommand(commandName: string, commandValue: string): void {
    this.command.emit({[commandName]: commandValue});
  }

}
