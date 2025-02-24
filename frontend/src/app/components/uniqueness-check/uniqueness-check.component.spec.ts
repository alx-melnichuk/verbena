import { ComponentFixture, TestBed } from '@angular/core/testing';

import { UniquenessCheckComponent } from './uniqueness-check.component';

describe('UniquenessCheckComponent', () => {
  let component: UniquenessCheckComponent;
  let fixture: ComponentFixture<UniquenessCheckComponent>;

  beforeEach(() => {
    TestBed.configureTestingModule({
      imports: [UniquenessCheckComponent]
    });
    fixture = TestBed.createComponent(UniquenessCheckComponent);
    component = fixture.componentInstance;
    fixture.detectChanges();
  });

  it('should create', () => {
    expect(component).toBeTruthy();
  });
});
