import { ChangeDetectionStrategy, Component, ViewEncapsulation } from '@angular/core';

@Component({
  selector: 'app-panel-banned-users',
  standalone: true,
  imports: [],
  templateUrl: './panel-banned-users.component.html',
  styleUrl: './panel-banned-users.component.scss',
  encapsulation: ViewEncapsulation.None,
  changeDetection: ChangeDetectionStrategy.OnPush
})
export class PanelBannedUsersComponent {

}
