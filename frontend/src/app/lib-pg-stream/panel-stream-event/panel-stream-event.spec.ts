import { ComponentFixture, TestBed } from '@angular/core/testing';

import { PanelStreamEvent } from './panel-stream-event';

describe('PanelStreamEvent', () => {
  let component: PanelStreamEvent;
  let fixture: ComponentFixture<PanelStreamEvent>;

  beforeEach(async () => {
    await TestBed.configureTestingModule({
      imports: [PanelStreamEvent]
    })
    .compileComponents();

    fixture = TestBed.createComponent(PanelStreamEvent);
    component = fixture.componentInstance;
    fixture.detectChanges();
  });

  it('should create', () => {
    expect(component).toBeTruthy();
  });
});
