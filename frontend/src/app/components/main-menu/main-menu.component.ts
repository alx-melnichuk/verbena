import { ChangeDetectionStrategy, Component, EventEmitter, Input, OnInit, Output, ViewEncapsulation } from '@angular/core';
import { CommonModule } from '@angular/common';
import { FormsModule } from '@angular/forms';
import { MatButtonModule } from '@angular/material/button';
import { MatMenuModule } from '@angular/material/menu';
import { MatSlideToggleModule } from '@angular/material/slide-toggle';
import { TranslateModule } from '@ngx-translate/core';
import { InitializationService } from 'src/app/common/initialization.service';


@Component({
  selector: 'app-main-menu',
  standalone: true,
  imports: [CommonModule, FormsModule, MatMenuModule, MatButtonModule, MatSlideToggleModule, TranslateModule],
  templateUrl: './main-menu.component.html',
  styleUrls: ['./main-menu.component.scss'],
  encapsulation: ViewEncapsulation.None,
  changeDetection: ChangeDetectionStrategy.OnPush,
})
export class MainMenuComponent implements OnInit {
  @Input()
  public isDisabledMenu = false;
  @Input()
  public isShowDarkTheme = false;
  @Input()
  public isShowLogout = false;


  @Output()
  readonly setDarkTheme: EventEmitter<boolean> = new EventEmitter();
  @Output()
  readonly logout: EventEmitter<void> = new EventEmitter();
  
  public isDarkTheme = false;

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
