import { ChangeDetectionStrategy, Component, EventEmitter, Input, OnInit, Output, ViewEncapsulation } from '@angular/core';
import { CommonModule } from '@angular/common';
import { FormsModule } from '@angular/forms';
import { MatButtonModule } from '@angular/material/button';
import { MatMenuModule } from '@angular/material/menu';
import { MatSlideToggleModule } from '@angular/material/slide-toggle';
import { TranslateModule } from '@ngx-translate/core';
import { RouterLink } from '@angular/router';

import { InitializationService } from 'src/app/common/initialization.service';
import { ROUTE_STREAM_LIST } from 'src/app/common/routes';


@Component({
  selector: 'app-main-menu',
  standalone: true,
  imports: [CommonModule, FormsModule, MatMenuModule, MatButtonModule, MatSlideToggleModule, TranslateModule,  RouterLink],
  templateUrl: './main-menu.component.html',
  styleUrls: ['./main-menu.component.scss'],
  encapsulation: ViewEncapsulation.None,
  changeDetection: ChangeDetectionStrategy.OnPush,
})
export class MainMenuComponent implements OnInit {
  @Input()
  public isDisabledMenu: boolean = false;
  @Input()
  public isShowMyStreams: boolean = false;
  @Input()
  public isShowDarkTheme: boolean = false;
  @Input()
  public isShowLogout: boolean = false;


  @Output()
  readonly setDarkTheme: EventEmitter<boolean> = new EventEmitter();
  @Output()
  readonly logout: EventEmitter<void> = new EventEmitter();
  
  public isDarkTheme = false;
  public linkMyStreams = ROUTE_STREAM_LIST;

  constructor(public initializationService: InitializationService) {
  }
  
  ngOnInit(): void {
    this.isDarkTheme = this.initializationService.getDarkTheme();
  }

  // **Public API **

  public doChangeDarkTheme(value: boolean): void {
    this.setDarkTheme.emit(value);
    this.isDarkTheme = this.initializationService.getDarkTheme();
  }

  public doLogout(): void {
    this.logout.emit();
  }
}
