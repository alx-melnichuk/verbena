import { ChangeDetectionStrategy, Component, ViewEncapsulation } from '@angular/core';
import { CommonModule } from '@angular/common';

@Component({
  selector: 'app-panel-live-streams',
  standalone: true,
  imports: [CommonModule],
  templateUrl: './panel-live-streams.component.html',
  styleUrls: ['./panel-live-streams.component.scss'],
  encapsulation: ViewEncapsulation.None,
  changeDetection: ChangeDetectionStrategy.OnPush
})
export class PanelLiveStreamsComponent {

}
