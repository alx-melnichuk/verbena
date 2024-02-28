import {
  AfterViewInit, ChangeDetectionStrategy, Component, EventEmitter, Input, OnChanges, OnDestroy, OnInit, Output,
  SimpleChanges, ViewChild, ViewEncapsulation
} from '@angular/core';
import { CommonModule } from '@angular/common';
import { MatCalendar, MatCalendarCellClassFunction, MatDatepickerModule } from '@angular/material/datepicker';
import { MatNativeDateModule } from '@angular/material/core';
import { StringDateTime } from 'src/app/common/string-date-time';
import { Subscription } from 'rxjs';

export const PSC_DAY_WITH_STREAMS = 'psc-day-with-streams';

@Component({
  selector: 'app-panel-stream-calendar',
  standalone: true,
  imports: [CommonModule, MatNativeDateModule, MatDatepickerModule],
  templateUrl: './panel-stream-calendar.component.html',
  styleUrls: ['./panel-stream-calendar.component.scss'],
  encapsulation: ViewEncapsulation.None,
  changeDetection: ChangeDetectionStrategy.OnPush
})
export class PanelStreamCalendarComponent implements OnInit, OnChanges, AfterViewInit, OnDestroy {

  @Input()
  public selected: Date | null = null;
  @Input()
  public markedDates: StringDateTime[] = [];
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

  constructor() {
    const now = new Date();
    now.setHours(0, 0, 0, 0);
    this.startAtDate = now;
    this.activeDate = now;
  }
  
  public ngOnChanges(changes: SimpleChanges): void {
    if (!!changes['markedDates'] && !!this.markedDates && !!this.calendar) {
      this.markedDatesStr = this.markedDates.map(val => this.getInfoDate(new Date(val)));
      this.calendar.updateTodaysDate();
    }
  }

  public ngOnInit(): void {
  }

  public ngAfterViewInit(): void {
    if (!!this.calendar) {
      this.activeDate = new Date(this.calendar.activeDate);
      this.stateChangesSub = this.calendar.stateChanges
        .subscribe((value) => this.changeSateCalendar());
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
  public dateClassFn: MatCalendarCellClassFunction<Date> = (date: Date, view: 'month' | 'year' | 'multi-year') => {
    let result: string = '';
    // Only highlight dates inside the month view.
    if (view === 'month') {
      const value = this.getInfoDate(date);
      if (this.markedDatesStr.includes(value)) {
        result = PSC_DAY_WITH_STREAMS;
      }
    }
    return result;
  };

  // ** Private API **

  private changeSateCalendar = (): void => {
    if (!this.calendar) {
      return;
    }
    const newActiveDate: Date = new Date(this.calendar.activeDate);
    const newActiveDateYearMonth = this.getInfoDate(newActiveDate).slice(0, 7);
    // console.log(`^^newActiveDate:`, newActiveDate); // #

    const currActiveDateYearMonth = this.getInfoDate(this.activeDate).slice(0, 7);
    if (!!newActiveDate && currActiveDateYearMonth != newActiveDateYearMonth) {
      const activeDate = new Date(newActiveDate.getFullYear(), newActiveDate.getMonth(), 1, 0, 0, 0, 0);
      this.activeDate = activeDate;
      // console.log(`^^activeDate:`, activeDate); // #
      this.changeCalendar.emit(new Date(activeDate));
    }
  }

  private getInfoDate(date: Date): string {
    const year = date.getFullYear();
    const month = ('00' + (date.getMonth() + 1)).slice(-2);
    const day = ('00' + date.getDate()).slice(-2);
    return `${year}-${month}-${day}`;
  }
}
