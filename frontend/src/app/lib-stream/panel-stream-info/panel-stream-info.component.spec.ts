import { ComponentFixture, TestBed } from '@angular/core/testing';

import { PanelStreamInfoComponent } from './panel-stream-info.component';

describe('PanelStreamInfoComponent', () => {
  let component: PanelStreamInfoComponent;
  let fixture: ComponentFixture<PanelStreamInfoComponent>;

  beforeEach(() => {
    TestBed.configureTestingModule({
      imports: [PanelStreamInfoComponent]
    });
    fixture = TestBed.createComponent(PanelStreamInfoComponent);
    component = fixture.componentInstance;
    fixture.detectChanges();
  });

  it('should create', () => {
    expect(component).toBeTruthy();
  });
});
