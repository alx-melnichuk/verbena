import { ComponentFixture, TestBed } from '@angular/core/testing';

import { PanelStreamStateComponent } from './panel-stream-state.component';

describe('PanelStreamStateComponent', () => {
  let component: PanelStreamStateComponent;
  let fixture: ComponentFixture<PanelStreamStateComponent>;

  beforeEach(() => {
    TestBed.configureTestingModule({
      imports: [PanelStreamStateComponent]
    });
    fixture = TestBed.createComponent(PanelStreamStateComponent);
    component = fixture.componentInstance;
    fixture.detectChanges();
  });

  it('should create', () => {
    expect(component).toBeTruthy();
  });
});
