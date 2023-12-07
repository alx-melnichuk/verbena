import { ChangeDetectionStrategy, Component, EventEmitter, Output, ViewEncapsulation } from '@angular/core';
import { CommonModule } from '@angular/common';
import { MatButtonModule } from '@angular/material/button';
import { MatMenuModule } from '@angular/material/menu';
import { TranslateModule } from '@ngx-translate/core';


@Component({
  selector: 'app-main-menu',
  standalone: true,
  imports: [CommonModule, MatMenuModule, MatButtonModule, TranslateModule],
  templateUrl: './main-menu.component.html',
  styleUrls: ['./main-menu.component.scss'],
  encapsulation: ViewEncapsulation.None,
  changeDetection: ChangeDetectionStrategy.OnPush,
})
export class MainMenuComponent {
  @Output()
  readonly logout: EventEmitter<void> = new EventEmitter();
  
  public isMenu = true;
  public isShowMyProfile = false;

  public doLogout(): void {
    this.logout.emit();
  }
}
