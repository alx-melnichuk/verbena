import { ChangeDetectionStrategy, Component, ViewEncapsulation } from '@angular/core';
import { CommonModule } from '@angular/common';

@Component({
  selector: 'app-panel-view-list',
  standalone: true,
  imports: [CommonModule],
  templateUrl: './panel-view-list.component.html',
  styleUrls: ['./panel-view-list.component.scss'],
  encapsulation: ViewEncapsulation.None,
  changeDetection: ChangeDetectionStrategy.OnPush
})
export class PanelViewListComponent {

}
