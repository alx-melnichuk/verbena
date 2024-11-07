import { ChangeDetectionStrategy, Component, Input, OnChanges, SimpleChanges, ViewEncapsulation } from '@angular/core';
import { CommonModule } from '@angular/common';
import { StringDateTimeUtil } from 'src/app/utils/string-date-time.util';

@Component({
  selector: 'app-panel-stream-params',
  exportAs: 'appPanelStreamParams',
  standalone: true,
  imports: [CommonModule],
  templateUrl: './panel-stream-params.component.html',
  styleUrls: ['./panel-stream-params.component.scss'],
  encapsulation: ViewEncapsulation.None,
  changeDetection: ChangeDetectionStrategy.OnPush
})
export class PanelStreamParamsComponent implements OnChanges {
  @Input()
  public title: string | null | undefined;
  @Input()
  public tags: string[] = [];
  @Input()
  public startDate: Date | null | undefined;

  public innStartDate: string | null = null;

  ngOnChanges(changes: SimpleChanges): void {
    if (!!changes['startDate']) {
      this.innStartDate = this.startDate != null ? this.startDate.toJSON() : '';
    }
  }
}
