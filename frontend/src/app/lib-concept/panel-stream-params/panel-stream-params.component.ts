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
  public startDateTime: Date | null | undefined;
  @Input()
  public countOfViewer: number | null | undefined;

  public innStartDate: string | null = null;
  public innStartTime: string | null = null;

  ngOnChanges(changes: SimpleChanges): void {
    if (!!changes['startDateTime']) {
      this.innStartDate = '';
      this.innStartTime = '';
      if (this.startDateTime != null) {
        this.innStartDate = this.startDateTime.toISOString().slice(0,10);
        this.innStartTime = this.startDateTime.toTimeString().slice(0,5);
      }
    }
  }
}
