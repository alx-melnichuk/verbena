import { ComponentFixture, TestBed } from '@angular/core/testing';

import { PanelStreamCalendar } from './panel-stream-calendar';

describe('PanelStreamCalendar', () => {
  let component: PanelStreamCalendar;
  let fixture: ComponentFixture<PanelStreamCalendar>;

  beforeEach(async () => {
    await TestBed.configureTestingModule({
      imports: [PanelStreamCalendar]
    })
    .compileComponents();

    fixture = TestBed.createComponent(PanelStreamCalendar);
    component = fixture.componentInstance;
    fixture.detectChanges();
  });

  it('should create', () => {
    expect(component).toBeTruthy();
  });
});
