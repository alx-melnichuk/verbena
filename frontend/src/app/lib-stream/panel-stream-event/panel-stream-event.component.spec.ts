import { ComponentFixture, TestBed } from '@angular/core/testing';

import { PanelStreamEventComponent } from './panel-stream-event.component';

describe('PanelStreamEventComponent', () => {
  let component: PanelStreamEventComponent;
  let fixture: ComponentFixture<PanelStreamEventComponent>;

  beforeEach(() => {
    TestBed.configureTestingModule({
      imports: [PanelStreamEventComponent]
    });
    fixture = TestBed.createComponent(PanelStreamEventComponent);
    component = fixture.componentInstance;
    fixture.detectChanges();
  });

  it('should create', () => {
    expect(component).toBeTruthy();
  });
});
