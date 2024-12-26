import {
  AfterViewInit, ChangeDetectionStrategy, Component, ElementRef, EventEmitter, Input, OnChanges, OnDestroy, OnInit, Output,
  SimpleChanges, ViewChild, ViewEncapsulation
} from '@angular/core';
import { CommonModule } from '@angular/common';
import { MatCalendar, MatCalendarCellClassFunction, MatDatepickerModule } from '@angular/material/datepicker';
import { MatNativeDateModule } from '@angular/material/core';
import { Subscription } from 'rxjs';

import { StringDateTime } from 'src/app/common/string-date-time';
import { HtmlElemUtil } from 'src/app/utils/html-elem.util';
import { StringDateTimeUtil } from 'src/app/utils/string-date-time.util';

import { StreamsPeriodDto } from '../stream-api.interface';

export const PSC_DAY_WITH_STREAMS = 'psc-day-with-streams';
export const PSC_DAY = '---psc-day-';

type PeriodMapType = {[key: string]: number};

@Component({
  selector: 'app-panel-stream-calendar',
  standalone: true,
  imports: [CommonModule, MatNativeDateModule, MatDatepickerModule],
  templateUrl: './panel-stream-calendar.component.html',
  styleUrls: ['./panel-stream-calendar.component.scss', 'panel-stream-calendar-emblem.component.scss'],
  encapsulation: ViewEncapsulation.None,
  changeDetection: ChangeDetectionStrategy.OnPush
})
export class PanelStreamCalendarComponent implements OnInit, OnChanges, AfterViewInit, OnDestroy {

  @Input()
  public selected: Date | null = null;
  @Input()
  public markedDates: StreamsPeriodDto[] = [];
  @Input()
  public minDate: Date | null = null;
  @Input()
  public maxDate: Date | null = null;

  @Output()
  readonly changeSelected: EventEmitter<Date | null> = new EventEmitter();
  @Output()
  readonly changeCalendar: EventEmitter<Date> = new EventEmitter();

  @ViewChild('calendar')
  public calendar: MatCalendar<Date> | null = null;

  public startAtDate: Date;
  public activeDate: Date;

  private stateChangesSub: Subscription | undefined;
  private markedDatesStr: string[] = [];
  private markedPeriodMap: PeriodMapType = {};

  constructor(
    public hostRef: ElementRef<HTMLElement>,
  ) {
    const now = new Date();
    now.setHours(0, 0, 0, 0);
    this.startAtDate = now;
    this.activeDate = now;
  }
  
  public ngOnChanges(changes: SimpleChanges): void {
    if (!!changes['markedDates'] && !!this.markedDates && !!this.calendar) {
      this.settingPropertiesByPeriodDate(this.hostRef, this.markedPeriodMap, false);
      this.markedPeriodMap = this.preparePeriodDate(this.markedDates);
      this.calendar.updateTodaysDate();
      this.settingPropertiesByPeriodDate(this.hostRef, this.markedPeriodMap, true);
    }
  }
  
  public ngOnInit(): void {
  }

  public ngAfterViewInit(): void {
    if (!!this.calendar) {
      this.activeDate = new Date(this.calendar.activeDate);
      this.stateChangesSub = this.calendar.stateChanges
        .subscribe(() => this.changeSateCalendar());
    }
  }

  public ngOnDestroy(): void {
    this.stateChangesSub?.unsubscribe();
  }
  
  // ** Public API **

  public doChangeSelected(value: Date | null): void {
    if (!!value) {
      this.changeSelected.emit(value);
    }
  }
  // Function that can be used to add custom CSS classes to dates.
  public dateClassFn: MatCalendarCellClassFunction<Date> = (date: Date, view: 'month' | 'year' | 'multi-year'): string[] => {
    let result: string[] = [];
    // Only highlight dates inside the month view.
    if (view === 'month') {
      const value = this.getInfoDate(date);
      if (this.markedDatesStr.includes(value)) {
        result.push(PSC_DAY_WITH_STREAMS);
      }
      const itemLocal: StringDateTime = StringDateTimeUtil.toISOLocal(date);
      const itemCount = this.markedPeriodMap[itemLocal];
      if (itemCount != null) {
        result = [PSC_DAY_WITH_STREAMS];
        result.push('psc-day-' + ('00' + date.getDate()).slice(-2));
      }
    }
    return result;
  };

  public changeSateCalendar = (): void => {
    if (!this.calendar) {
      return;
    }
    const newActiveDate: Date = new Date(this.calendar.activeDate);
    const newActiveDateYearMonth = this.getInfoDate(newActiveDate).slice(0, 7);

    const currActiveDateYearMonth = this.getInfoDate(this.activeDate).slice(0, 7);
    if (!!newActiveDate && currActiveDateYearMonth != newActiveDateYearMonth) {
      const activeDate = new Date(newActiveDate.getFullYear(), newActiveDate.getMonth(), 1, 0, 0, 0, 0);
      this.activeDate = activeDate;
      this.changeCalendar.emit(new Date(activeDate));
    }
  }

  // ** Private API **

  private getInfoDate(date: Date): string {
    const year = date.getFullYear();
    const month = ('00' + (date.getMonth() + 1)).slice(-2);
    const day = ('00' + date.getDate()).slice(-2);
    return `${year}-${month}-${day}`;
  }

  private preparePeriodDate(list: StreamsPeriodDto[]): PeriodMapType {
    const result: PeriodMapType = {};
    for (let idx = 0; idx < list.length; idx++) {
      const itemDate: Date | null = new Date(list[idx].date);
      if (!itemDate) continue;
      itemDate.setHours(0, 0, 0, 0);
      const itemLocal: StringDateTime = StringDateTimeUtil.toISOLocal(itemDate);
      result[itemLocal] = (result[itemLocal] || 0) + list[idx].count;
    }
    return result;
  }
  private settingPropertiesByPeriodDate(elem: ElementRef<HTMLElement> | null, periodMap: PeriodMapType, isSetValue: boolean): void {
    const list: string[] = Object.keys(periodMap);
    for (let idx = 0; idx < list.length; idx++) {
      const value = isSetValue ? periodMap[list[idx]].toString() : null;
      HtmlElemUtil.setProperty(elem, PSC_DAY + list[idx].slice(8,10), value);
    }
  }
}
