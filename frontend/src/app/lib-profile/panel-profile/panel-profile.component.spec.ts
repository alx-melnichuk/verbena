import { ComponentFixture, TestBed } from '@angular/core/testing';

import { PanelProfileComponent } from './panel-profile.component';

describe('PanelProfileComponent', () => {
  let component: PanelProfileComponent;
  let fixture: ComponentFixture<PanelProfileComponent>;

  beforeEach(() => {
    TestBed.configureTestingModule({
      imports: [PanelProfileComponent]
    });
    fixture = TestBed.createComponent(PanelProfileComponent);
    component = fixture.componentInstance;
    fixture.detectChanges();
  });

  it('should create', () => {
    expect(component).toBeTruthy();
  });
});
