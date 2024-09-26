import { ChangeDetectionStrategy, Component, EventEmitter, Input, OnChanges, OnInit, Output, SimpleChanges, ViewEncapsulation } from '@angular/core';
import { CommonModule } from '@angular/common';
import { FormsModule } from '@angular/forms';
import { MatButtonModule } from '@angular/material/button';
import { MatMenuModule } from '@angular/material/menu';
import { MatSlideToggleModule } from '@angular/material/slide-toggle';
import { TranslateModule, TranslateService } from '@ngx-translate/core';
import { RouterLink } from '@angular/router';

import { InitializationService } from 'src/app/common/initialization.service';
import { ROUTE_STREAM_CREATE, ROUTE_STREAM_LIST } from 'src/app/common/routes';
import { LOCALE_DE_DE, LOCALE_EN_US, LOCALE_UK, THEME_DARK, THEME_LIGHT } from 'src/app/common/constants';


@Component({
  selector: 'app-main-menu',
  standalone: true,
  imports: [CommonModule, FormsModule, MatMenuModule, MatButtonModule, MatSlideToggleModule, TranslateModule,  RouterLink],
  templateUrl: './main-menu.component.html',
  styleUrls: ['./main-menu.component.scss'],
  encapsulation: ViewEncapsulation.None,
  changeDetection: ChangeDetectionStrategy.OnPush,
})
export class MainMenuComponent implements OnInit, OnChanges {
  @Input()
  public isDisabledMenu: boolean = false;
  @Input()
  public isShowMyStreams: boolean = false;
  @Input()
  public isShowCreateStream: boolean = false;
  @Input()
  public isShowTheme: boolean = false;
  @Input()
  public isShowLocale: boolean = false;
  @Input()
  public isShowLogout: boolean = false;
  @Input()
  public locale: string | null = null;
  @Input()
  public theme: string | null = null;

  @Output()
  readonly logout: EventEmitter<void> = new EventEmitter();
  @Output()
  readonly setLocale: EventEmitter<string> = new EventEmitter();
  @Output()
  readonly setTheme: EventEmitter<string> = new EventEmitter();
  
  public isDarkTheme: boolean = false;
  public linkMyStreams = ROUTE_STREAM_LIST;
  public linkCreateStream = ROUTE_STREAM_CREATE;

  public localeList = [
    { value: '', name: this.translate.instant('profile.text_nothing') },
    { value: LOCALE_EN_US, name: this.translate.instant('profile.text_locale_en_us') },
    { value: LOCALE_DE_DE, name: this.translate.instant('profile.text_locale_de_de') },
    { value: LOCALE_UK, name: this.translate.instant('profile.text_locale_uk') },
  ];
  public localeMap: { [key: string]: string } = {};

  constructor(
    private translate: TranslateService,
    public initializationService: InitializationService,
  ) {
    for (let idx = 0; idx < this.localeList.length; idx++) {
        const item = this.localeList[idx];
        this.localeMap[item.value] = item.name;
    }
  }
  
  ngOnInit(): void {
  }

  ngOnChanges(changes: SimpleChanges): void {
    if (!!changes['theme']) {
      this.isDarkTheme = this.theme == THEME_DARK;
    }
  }

  // **Public API **

  public doLogout(): void {
    this.logout.emit();
  }

  public doSetLocale(value: string): void {
    this.setLocale.emit(value);
  }

  public doSetTheme(value: boolean): void {
    this.setTheme.emit(value ? THEME_DARK : THEME_LIGHT);
  }
}
