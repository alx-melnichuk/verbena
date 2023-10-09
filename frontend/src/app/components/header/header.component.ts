import { CommonModule } from '@angular/common';
import { ChangeDetectionStrategy, Component, EventEmitter, Input, Output, SimpleChanges, ViewEncapsulation } from '@angular/core';
import { RouterLink } from '@angular/router';
import { TranslateModule } from '@ngx-translate/core';
import { UserDto } from 'src/app/entities/user/user-dto';

@Component({
  selector: 'app-header',
  standalone: true,
  imports: [CommonModule, RouterLink, TranslateModule],
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

  constructor() {
    console.log(`HeaderComponent();`); // #-
  }

  ngOnChanges(changes: SimpleChanges): void {
    if (!!changes['userInfo']) {
      console.log(`changes['userInfo'] ${!!this.userInfo ? this.userInfo.nickname : 'null'}`); // #-
      // #- this.prepareData();
    }
  }

  // ** Public API **

  public doLogout(): void {
    this.logout.emit();
  }
}
